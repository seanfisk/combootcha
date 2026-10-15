//! Parser and emitter for RetroArch's `retroarch.cfg`, based on these sources:
//!
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/include/file/config_file.h
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/file/config_file.c
//!
//! The grammar has been updated in [this pull request](https://github.com/libretro/RetroArch/pull/19218). This will very likely be included in a future release, but as of time of writing, 2026-10, we will target the grammar in version 1.22.2.
//!
//! Emulated libretro behavior:
//!
//! - Lines are split on `\n`
//! - A line starting with `#` in column 0 is either a `#include "path"` / `#reference "path"` directive or a comment.
//! - Otherwise, a line is `key = value`
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
//! Deliberate and known deviations from libretro behavior:
//!
//! - Repeated keys collapse to the topmost entry.
//! - Lines with an empty key or an empty `#reference` path are dropped.

use anyhow::{bail, Context, Result};
use log::debug;
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

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Config {
    entries: BTreeMap<Key, String>,
    includes: Vec<String>,
    references: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum Line {
    Ignored,
    Include(String),
    Reference(String),
    Entry(String, String),
}

/// Port of libretro's `ISSPACE` (a `lr_char_props` table lookup): space, tab, LF and CR only. Unlike C `isspace`, it excludes `\f` and `\v`.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | terminals::NEWLINE | '\r')
}

/// Port of `config_file_extract_value`.
fn extract_value(s: &str) -> String {
    let s = s.trim_start_matches(is_space);
    if let Some(rest) = s.strip_prefix(terminals::QUOTE) {
        // Up to the next quote, or to the end of the line if unterminated.
        rest.split(terminals::QUOTE)
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        s.chars().take_while(char::is_ascii_graphic).collect()
    }
}

/// Port of the non-directive path of `config_file_strip_comment`: cut at the first `#` unless it lies inside the first quoted literal.
fn strip_comment(line: &str) -> &str {
    let Some(hash) = line.find(terminals::COMMENT_START) else {
        return line;
    };
    if let Some(open) = line.find(terminals::QUOTE) {
        if open < hash {
            if let Some(close_rel) = line[open + 1..].find(terminals::QUOTE) {
                if open + 1 + close_rel > hash {
                    return line;
                }
            }
        }
    }
    &line[..hash]
}

fn parse_directive_path(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let path = extract_value(text);
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Port of `config_file_parse_line`. While `config_file_parse_line` modifies the struct fields directly and returns a Boolean indicating whether the line was meaningful, this version returns the parsed line without modifying any state.
fn parse_line(raw: &str) -> Line {
    if let Some(comment) = raw.strip_prefix(terminals::COMMENT_START) {
        if let Some(path) = comment.strip_prefix(terminals::INCLUDE_DIRECTIVE) {
            return parse_directive_path(path).map_or(Line::Ignored, Line::Include);
        } else if let Some(path) = comment.strip_prefix(terminals::REFERENCE_DIRECTIVE) {
            return parse_directive_path(path).map_or(Line::Ignored, Line::Reference);
        }
    }
    let line = strip_comment(raw).trim_start_matches(is_space);
    let key: String = line.chars().take_while(char::is_ascii_graphic).collect();
    if key.is_empty() {
        return Line::Ignored;
    }
    let rest = line[key.len()..].trim_start_matches(is_space);
    let Some(value) = rest.strip_prefix('=') else {
        return Line::Ignored; // No '=' after the key
    };
    Line::Entry(key, extract_value(value))
}

/// Keys are ASCII graphic runs. `#` would start a comment, and `"` could make a later `#` in the value look like it's outside the quoted literal.
fn check_key(key: &str) -> Result<()> {
    if key.is_empty()
        || !key.bytes().all(|b| b.is_ascii_graphic())
        || key.contains([terminals::COMMENT_START, terminals::QUOTE])
    {
        bail!("invalid RetroArch config key: {key:?}"); // TODO improve
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

fn write_directives<W, D, P>(mut writer: &mut W, directive: D, paths: P) -> Result<()>
where
    W: Write,
    D: Display,
    P: IntoIterator,
    P::Item: Display,
{
    for path in paths.into_iter() {
        writeln!(
            &mut writer,
            r#"{}{}"{}""#,
            terminals::COMMENT_START,
            directive,
            path
        )?;
    }
    Ok(())
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
                debug!("Adding new value {:?} for key {:?}", new_value, slot.key());
                slot.insert(new_value.to_string());
                true
            }
            Entry::Occupied(mut slot) => {
                let old_value = slot.get();
                if new_value == *old_value {
                    debug!(
                        "Desired value {:?} for key {:?} matches existing value; no need to update",
                        new_value,
                        slot.key()
                    );
                    false
                } else if replace {
                    debug!(
                        "Replacing existing value {:?} for key {:?} with desired value {:?}",
                        slot.get(),
                        slot.key(),
                        new_value,
                    );
                    slot.insert(new_value.into_owned());
                    true
                } else {
                    debug!(
                        "Discarding duplicate value {:?} for {:?}; keeping existing value {:?}",
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
                .with_context(|| format!("Cannot read line {}", line_number))?;
            if chars_read == 0 {
                break;
            }
            if line.ends_with(terminals::NEWLINE) {
                line.pop();
            }
            match parse_line(&line) {
                Line::Ignored => {}
                Line::Include(path) => config.includes.push(path),
                Line::Reference(path) => config.references.push(path),
                Line::Entry(key, value) => {
                    config
                        .add(key, value)
                        .with_context(|| format!("Cannot parse line {}", line_number))?;
                }
            }
            line.clear();
            line_number += 1;
        }
        Ok(config)
    }

    pub(crate) fn write_to<W: Write>(&self, writer: W) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        // TODO Add line numbers to errors?
        write_directives(
            &mut writer,
            terminals::REFERENCE_DIRECTIVE,
            &self.references,
        )?;
        for (key, value) in self.entries.iter() {
            writeln!(&mut writer, r#"{} = "{}""#, key.as_str(), value)?;
        }
        write_directives(&mut writer, terminals::INCLUDE_DIRECTIVE, &self.includes)?;
        writer.flush()?;
        Ok(())
    }
}
