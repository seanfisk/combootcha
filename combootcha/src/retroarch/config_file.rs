//! Parser and emitter for RetroArch's `retroarch.cfg`, based on these sources:
//!
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/include/file/config_file.h
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/file/config_file.c
//!
//! The grammar has been updated in [this pull request](https://github.com/libretro/RetroArch/pull/19218). This will very likely be included in a future release, but as of time of writing, 2026-10, we will target the grammar in version 1.22.2.
//!
//! Emulated libretro behavior:
//!
//! - Lines are split on `\n`.
//! - A line starting with `#` in column 0 is either a `#include "path"` directive, `#reference "path"` directive, or a comment.
//! - Otherwise, a line is `key = value`.
//! - A key is a run of ASCII graphic characters, followed by optional whitespace, `=`, and the value.
//! - A value is either `"quoted"` (ends at the next `"`; there is no escaping, and an unterminated quote runs to end of line) or unquoted (a run of ASCII graphic characters).
//! - A `#` after the start of the line begins a comment unless it falls inside the first quoted literal.
//! - The topmost entry wins when a key repeats.
//! - Lookups are case-sensitive.
//! - File is written in this order:
//!   1. `#reference` lines
//!   2. Entries sorted by keys with `strcasecmp`, with each entry as `key = "value"`.
//!   3. `#include` lines
//!
//! Deliberate deviations from libretro behavior:
//!
//! - Repeated keys collapse to the topmost entry. TODO is this true?
//! - Lines with an empty key or an empty `#reference` path are dropped. TODO confirm
//!
//! Comments copied from the libretro source code have been prefixed as such.

use anyhow::{bail, Context, Result};
use log::{debug, info};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

mod terminals {
    pub(super) const NEWLINE: char = '\n';
    pub(super) const INCLUDE_DIRECTIVE: &str = "include ";
    pub(super) const REFERENCE_DIRECTIVE: &str = "reference ";
    pub(super) const COMMENT_START: char = '#';
    pub(super) const QUOTE: char = '"';
}

/// A config key ordered like `strcasecmp` (ASCII case-insensitive), with a byte-wise tiebreak so keys differing only by case stay distinct.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key(String);

impl Key {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.0.bytes().map(|b| b.to_ascii_lowercase());
        let b = other.0.bytes().map(|b| b.to_ascii_lowercase());
        a.cmp(b).then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "key {:?}", self.0)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Config {
    entries: BTreeMap<Key, String>,
    includes: Vec<String>,
    references: Vec<String>,
}

/// Port of libretro's `ISSPACE` (a `lr_char_props` table lookup): space, tab, LF and CR only. Unlike C `isspace`, it excludes `\f` and `\v`.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | terminals::NEWLINE | '\r')
}

/// Port of `config_file_extract_value`.
fn extract_value(s: &str) -> String {
    let s = s.trim_start_matches(is_space);
    if let Some(rest) = s.strip_prefix(terminals::QUOTE) {
        // Up to the next quote, or to the end of the line if unterminated
        rest.split(terminals::QUOTE)
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        s.chars().take_while(char::is_ascii_graphic).collect()
    }
}

enum LineText<'a> {
    Entry(&'a str),
    Comment(&'a str),
}

/// Port of libretro's `config_file_strip_comment`. Returns entry text rather than mutating the argument.
///
/// Searches input string for a comment (*#*) entry
///
/// If first character is *#*, then entire line is a comment and may correspond to a directive (command action -- e.g. include sub-config file). In this case, the function returns the comment itself (everything after the *#* character)
/// If a *#* character is found inside a string literal value, then it does not correspond to a comment and is ignored. In this case, the whole line is entry text
/// If a *#* character is found anywhere else, then the comment text is a suffix of the input string and has no programmatic value. In this case, the entry text is returned with comment removed
fn split_line_text(line: &str) -> LineText<'_> {
    // libretro: Search for a comment character
    if let Some(comment_start) = line.find(terminals::COMMENT_START) {
        // libretro: Check whether entire line is a comment
        // > First character == '#'
        if comment_start == 0 {
            // Return comment for processing at a higher level
            return LineText::Comment(line);
        }
        // libretro: Comment character occurs at an offset:
        // Search for the start of a string literal value
        if let Some(literal_start) = line.find(terminals::QUOTE) {
            // libretro: Check whether string literal start occurs *before* the comment character
            if literal_start < comment_start {
                // libretro: Search for the end of the string literal value
                if let Some(literal_end) = line[literal_start + 1..].find(terminals::QUOTE) {
                    // libretro: Check whether string literal end occurs *after* the comment character
                    // > If this is the case, ignore the comment
                    // > Return the entry text
                    if literal_end > comment_start {
                        return LineText::Entry(line);
                    }
                }
            }
        }
        // libretro: If we reach this point, then a comment exists outside of a string literal
        // > Trim the entire comment from the end of the line
        return LineText::Entry(&line[..comment_start]);
    }

    LineText::Entry(line)
}

fn parse_directive_path(text: &str) -> Option<String> {
    // TODO libretro doesn't do this?
    // if text.is_empty() {
    //     return None;
    // }
    let path = extract_value(text);
    if path.is_empty() {
        // TODO libretro doesn't do this?
        None
    } else {
        Some(path)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LineContent {
    Ignored,
    Include(String),
    Reference(String),
    Entry(String, String),
}

/// Port of libretro's `config_file_parse_line`.
///
/// The C function modifies the struct fields directly and returns a `boolean` indicating whether the line contained an entry. Since this does not work well in Rust, this port returns the parsed line without mutating any state.
fn parse_line(raw: &str) -> LineContent {
    // libretro: Remove any comment text
    let line_text = split_line_text(raw);
    // libretro: Check whether entire line is a comment
    match line_text {
        LineText::Comment(text) => {
            if let Some(path) = text.strip_prefix(terminals::INCLUDE_DIRECTIVE) {
                parse_directive_path(path).map_or(LineContent::Ignored, LineContent::Include)
            } else if let Some(path) = text.strip_prefix(terminals::REFERENCE_DIRECTIVE) {
                parse_directive_path(path).map_or(LineContent::Ignored, LineContent::Reference)
            } else {
                // libretro: All comments except those starting with the include or reference directive are ignored

                // libretro handles sub-config files at this point in the function
                // We don't need to read those since we just round-trip the directives in the original file
                LineContent::Ignored
            }
        }
        LineText::Entry(text) => {
            // libretro: Skip to first non-space character
            let text = text.trim_start_matches(is_space);

            // libretro: Copy line contents into key until we reach the next space character
            let key: String = text.chars().take_while(char::is_ascii_graphic).collect();

            // libretro discards entries with empty keys *after* parsing the value, but this seems like a minor oversight rather than a deliberate design decision. We discard eagerly here.
            if key.is_empty() {
                return LineContent::Ignored;
            }

            // libretro: An entry without a value is invalid
            let text = text[key.len()..].trim_start_matches(is_space);

            if let Some(value) = text.strip_prefix('=') {
                LineContent::Entry(key, extract_value(value))
            } else {
                // libretro: If we don't have an equal sign here, we've got an invalid string.
                LineContent::Ignored
            }
        }
    }
}

/// Keys are ASCII graphic runs. `#` would start a comment, and `"` could make a later `#` in the value look like it's outside the quoted literal.
fn check_key(key: &str) -> Result<()> {
    if key.is_empty()
        || !key.bytes().all(|b| b.is_ascii_graphic())
        || key.contains([terminals::COMMENT_START, terminals::QUOTE])
    {
        bail!("Invalid RetroArch config key {key:?}");
    }
    Ok(())
}

/// There is no escaping, so values and paths can't hold `"` or a line break. `#include` / `#reference` paths are written raw, like values.
fn check_text(what: &str, text: &str) -> Result<()> {
    if text.contains([terminals::QUOTE, terminals::NEWLINE]) {
        bail!("{what} can't be represented in retroarch.cfg: {text:?}"); // TODO improve
    }
    Ok(())
}

fn write_directive<W, D, P>(mut writer: &mut W, directive: D, path: P) -> std::io::Result<()>
where
    W: Write,
    D: Display,
    P: Display,
{
    writeln!(
        &mut writer,
        r#"{}{}"{}""#,
        terminals::COMMENT_START,
        directive,
        path
    )
}

impl Config {
    /// Ensure `key` is set to `value`, returning whether the configuration was updated.
    pub(crate) fn replace<'a, K, V>(&mut self, key: K, value: V) -> Result<bool>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.set(key.into(), value.into(), /*replace=*/ true)
    }

    /// Add a key-value pair if one was not already present.
    pub(super) fn add<'a, K, V>(&mut self, key: K, value: V) -> Result<bool>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.set(key.into(), value.into(), /*replace=*/ false)
    }

    /// Attempt to associate a key with a new value, optional replacing it if already present. Returns a `bool` indicating whether the configuration was updated.
    fn set(&mut self, key: Cow<'_, str>, new_value: Cow<'_, str>, replace: bool) -> Result<bool> {
        // Design decision: eagerly validate keys and values even if the value will be discarded
        check_key(&key)?;
        check_text("value", &new_value)?;

        use std::collections::btree_map::Entry;
        Ok(match self.entries.entry(Key(key.into_owned())) {
            Entry::Vacant(slot) => {
                debug!("Adding new value {:?} for {}", new_value, slot.key());
                slot.insert(new_value.to_string());
                true
            }
            Entry::Occupied(mut slot) => {
                let old_value = slot.get();
                if new_value == *old_value {
                    debug!(
                        "Desired value {:?} for {} matches existing value; no need to update",
                        new_value,
                        slot.key()
                    );
                    false
                } else if replace {
                    info!(
                        "Replacing existing value {:?} for {} with desired value {:?}",
                        slot.get(),
                        slot.key(),
                        new_value,
                    );
                    slot.insert(new_value.into_owned());
                    true
                } else {
                    debug!(
                        "Discarding duplicate value {:?} for {}; keeping existing value {:?}",
                        new_value,
                        slot.key(),
                        slot.get()
                    );
                    false
                }
            }
        })
    }

    /// Streaming equivalent of `parse`. Splits on `\n` only, like libretro (`BufRead::lines` would also strip a `\r` before the `\n`).
    pub(crate) fn read_from<R: Read>(reader: R) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut config = Self::default();
        let mut line = String::new();
        let mut line_number = 1;
        loop {
            let chars_read = reader
                .read_line(&mut line)
                .with_context(|| format!("Cannot read config line {}", line_number))?;
            if chars_read == 0 {
                break;
            }
            if line.ends_with(terminals::NEWLINE) {
                line.pop();
            }
            match parse_line(&line) {
                LineContent::Ignored => {}
                LineContent::Include(path) => config.includes.push(path),
                LineContent::Reference(path) => config.references.push(path),
                LineContent::Entry(key, value) => {
                    config
                        .add(key, value)
                        .with_context(|| format!("Cannot parse config line {}", line_number))?;
                }
            }
            line.clear();
            line_number += 1;
        }
        Ok(config)
    }

    pub(crate) fn write_to<W: Write>(&self, writer: W) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        let mut line_number = 1;
        for path in self.references.iter() {
            write_directive(&mut writer, terminals::REFERENCE_DIRECTIVE, path).with_context(
                || format!("Cannot write reference directive at line {line_number}"),
            )?;
            line_number += 1;
        }
        for (key, value) in self.entries.iter() {
            writeln!(&mut writer, r#"{} = "{}""#, key.as_str(), value)
                .with_context(|| format!("Cannot write key-value entry at line {line_number}"))?;
            line_number += 1;
        }
        // libretro: Config files are read from the top down: if duplicate entries are found then the topmost one in the list takes precedence. This means `#include' directives must go *after* individual config entries, otherwise they will override any custom-set values
        for path in self.includes.iter() {
            write_directive(&mut writer, terminals::INCLUDE_DIRECTIVE, path)
                .with_context(|| format!("Cannot write include directive at line {line_number}"))?;
            line_number += 1;
        }
        writer.flush()?;
        Ok(())
    }
}
