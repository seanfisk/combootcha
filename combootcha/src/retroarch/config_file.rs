//! Parser and emitter for RetroArch's `retroarch.cfg`, based on these sources:
//!
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/include/file/config_file.h
//! - https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/file/config_file.c
//!
//! The grammar has been updated in [this pull request](https://github.com/libretro/RetroArch/pull/19218). This will very likely be included in a future release, but as of time of writing, 2026-10, we will target the grammar in version 1.22.2.
//!
//! Comments copied from the libretro source code have been prefixed as such.

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info, log, trace, warn, Level};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Config {
    entries: BTreeMap<Key, String>,
    includes: Vec<String>,
    references: Vec<String>,
}

impl Config {
    fn add_include<'a, P>(&mut self, path: P) -> Result<()>
    where
        P: Into<Cow<'a, str>>,
    {
        let path = path.into();
        validate_value(path.as_ref())?;
        self.includes.push(path.into_owned());
        Ok(())
    }

    fn add_reference<'a, P>(&mut self, path: P) -> Result<()>
    where
        P: Into<Cow<'a, str>>,
    {
        let path = path.into();
        validate_value(path.as_ref())?;
        self.references.push(path.into_owned());
        Ok(())
    }

    // Generics are added to public functions, whereas the types in the private `set` function is concrete. This ensures that concrete copies generated for the generic remain small and is a common Rust pattern.

    /// Ensure `key` is set to `value`, returning whether the configuration was updated.
    pub(crate) fn replace<'a, K, V>(&mut self, key: K, value: V) -> Result<bool>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.set(key.into(), value.into(), /*replace=*/ true)
    }

    /// Add a key-value pair if one was not already present.
    fn add<'a, K, V>(&mut self, key: K, value: V) -> Result<bool>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        // libretro: Only add entry to the map if an entry with the specified value does not already exist
        self.set(key.into(), value.into(), /*replace=*/ false)
    }

    /// Attempt to associate a key with a new value, optional replacing it if already present. Returns a `bool` indicating whether the configuration was updated.
    fn set(&mut self, key: Cow<'_, str>, new_value: Cow<'_, str>, replace: bool) -> Result<bool> {
        // Design decision: eagerly validate keys and values even if the value will be discarded

        trace!("Validating RetroArch config key {key:?}");
        // Keys are ASCII graphic runs. `#` would start a comment, and `"` could make a later `#` in the value look like it's outside the quoted literal.
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_graphic())
            || key.contains([terminals::COMMENT_START, terminals::QUOTE])
        {
            bail!("Invalid RetroArch config key {key:?}");
        }

        validate_value(new_value.as_ref())?;

        use std::collections::btree_map::Entry;
        Ok(match self.entries.entry(Key(key.into_owned())) {
            Entry::Vacant(slot) => {
                // If replacing, it helps to know that the key wasn't there already. Otherwise it can be somewhat misleading why the file is being written in the caller.
                log!(
                    if replace { Level::Info } else { Level::Debug },
                    "Adding new value {:?} for {}",
                    new_value,
                    slot.key()
                );
                slot.insert(new_value.into_owned());
                true
            }
            Entry::Occupied(mut slot) => {
                let old_value = slot.get();
                if new_value == *old_value {
                    debug!(
                        "New value {:?} for {} matches existing value; no need to update",
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
                    warn!(
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

    pub(crate) fn read_from<R: Read>(reader: R) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut config = Self::default();
        let mut line = String::new();
        let mut line_number = 1;
        loop {
            let chars_read = reader
                .read_line(&mut line)
                .with_context(|| format!("Cannot read config line {line_number}"))?;
            if chars_read == 0 {
                break;
            }
            if line.ends_with(terminals::NEWLINE) {
                line.pop();
            }
            match parse_line(&line) {
                LineContent::Ignored => {
                    trace!("Ignoring line {line_number}");
                    Ok(())
                }
                LineContent::Malformed(reason) => {
                    warn!("Ignoring malformed line {line_number}: {reason}");
                    Ok(())
                }
                LineContent::Include(path) => {
                    debug!("Adding include path {path:?}");
                    config.add_include(path)
                }
                LineContent::Reference(path) => {
                    debug!("Adding reference path {path:?}");
                    config.add_reference(path)
                }
                LineContent::Entry(key, value) => config.add(key, value).map(|_| ()),
            }
            .with_context(|| format!("Cannot parse config line {line_number}"))?;
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
        // libretro: Config files are read from the top down - if duplicate entries are found then the topmost one in the list takes precedence. This means `#include' directives must go *after* individual config entries, otherwise they will override any custom-set values
        for path in self.includes.iter() {
            write_directive(&mut writer, terminals::INCLUDE_DIRECTIVE, path)
                .with_context(|| format!("Cannot write include directive at line {line_number}"))?;
            line_number += 1;
        }
        writer.flush()?;
        Ok(())
    }
}

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

/// Port of libretro's `ISSPACE` (a `lr_char_props` table lookup): space, tab, LF and CR only. Unlike C `isspace`, it excludes `\f` and `\v`.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | terminals::NEWLINE | '\r')
}

/// Port of libretro's `config_file_extract_value`.
fn extract_value(line: &str) -> String {
    let line = line.trim_start_matches(is_space);
    // libretro: Note: From this point on, an empty value string is valid
    if let Some(rest) = line.strip_prefix(terminals::QUOTE) {
        // libretro: If first character is ("), we have a full string literal

        // Read up to the next quote, or to the end of the line if the literal is unterminated
        rest.split(terminals::QUOTE)
            .next()
            .unwrap_or("") // libretro: If this a ("), then value string is empty
            .to_string()
    } else {
        // libretro: This is not a string literal - just read until the next space is found

        // libretro 1.22.2 introduces a round-trip bug here by not rejecting quotes embedded in an unquoted value. This results in this series of actions:
        //
        // Given: #reference a"b
        // Reads path: a"b
        // Writes: #reference "a"b"
        //
        // Given: #reference "a"b"
        // Reads path: a
        // Writes: #reference "a"
        //
        // Given: #reference "a"
        // Reads path: a
        // Writes: #reference "a"
        //
        // The cycle then stabilizes.
        //
        // We reject all values with embedded quotes or newlines in the mutators of Config to ensure that doesn't happen. We deliberately do not validate here in order to mirror the structure of this function in libretro.
        line.chars().take_while(char::is_ascii_graphic).collect()
    }
}

fn validate_value(text: &str) -> Result<()> {
    trace!("Validating RetroArch value {text:?}");
    // There is no escaping, so values and paths cannot hold `"` or a line break.
    if text.contains([terminals::QUOTE, terminals::NEWLINE]) {
        Err(anyhow!("Invalid RetroArch value {text:?}"))
    } else {
        Ok(())
    }
}

enum LineText<'a> {
    Entry(&'a str),
    Comment(&'a str),
}

/// Port of libretro's `config_file_strip_comment`. Returns entry text rather than mutating the argument.
///
/// Searches input string for a comment (`#`) entry
///
/// If first character is `#`, then entire line is a comment and may correspond to a directive (command action -- e.g. include sub-config file). In this case, the function returns the comment itself (everything after the `#` character)
/// If a `#` character is found inside a string literal value, then it does not correspond to a comment and is ignored. In this case, the whole line is entry text
/// If a `#` character is found anywhere else, then the comment text is a suffix of the input string and has no programmatic value. In this case, the entry text is returned with comment removed
fn split_line_text(line: &str) -> LineText<'_> {
    // libretro: Search for a comment character
    if let Some(comment_start) = line.find(terminals::COMMENT_START) {
        // libretro: Check whether entire line is a comment
        // > First character == '#'
        if comment_start == 0 {
            // Return comment for processing at a higher level
            return LineText::Comment(&line[1..]);
        }
        // libretro: Comment character occurs at an offset:
        // Search for the start of a string literal value
        if let Some(literal_start) = line.find(terminals::QUOTE) {
            // libretro: Check whether string literal start occurs *before* the comment character
            if literal_start < comment_start {
                // libretro: Search for the end of the string literal value
                if let Some(relative_literal_end) = line[literal_start + 1..].find(terminals::QUOTE)
                {
                    // libretro: Check whether string literal end occurs *after* the comment character
                    // > If this is the case, ignore the comment
                    // > Return the entry text
                    let literal_end = literal_start + 1 + relative_literal_end;
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

enum LineContent {
    Include(String),
    Reference(String),
    Entry(String, String),
    Malformed(&'static str),
    Ignored,
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
                if path.is_empty() {
                    LineContent::Malformed("missing include path")
                } else {
                    let path = extract_value(path);
                    if path.is_empty() {
                        LineContent::Malformed("empty include path")
                    } else {
                        LineContent::Include(path)
                    }
                }
                // libretro handles sub-config files at this point in the function
                // We don't need to read those since we just round-trip the directives in the original file
            } else if let Some(path) = text.strip_prefix(terminals::REFERENCE_DIRECTIVE) {
                if path.is_empty() {
                    LineContent::Malformed("missing reference path")
                } else {
                    // libretro quirk: references with empty string paths are retained. Emulate.
                    LineContent::Reference(extract_value(path))
                }
            } else {
                // libretro: All comments except those starting with the include or reference directive are ignored

                // Notably, the "include or reference directive" *includes a trailing space*, so the lines `#include` or `#reference` will *not* be classified by this code as directives, and will be ignored as plain comments
                LineContent::Ignored
            }
        }
        LineText::Entry(text) => {
            // libretro: Skip to first non-space character
            let text = text.trim_start_matches(is_space);

            // libretro: Copy line contents into key until we reach the next space character
            let key: String = text.chars().take_while(char::is_ascii_graphic).collect();

            // libretro ignores entries with empty keys *after* parsing the value, but this seems like a minor oversight rather than a deliberate design decision. We ignore eagerly here.
            if key.is_empty() {
                return LineContent::Ignored;
            }

            // libretro: An entry without a value is invalid
            let text = text[key.len()..].trim_start_matches(is_space);

            if let Some(value) = text.strip_prefix('=') {
                LineContent::Entry(key, extract_value(value))
            } else {
                // libretro: If we don't have an equal sign here, we've got an invalid string.
                LineContent::Malformed("missing value")
            }
        }
    }
}

fn write_directive<W, D, P>(mut writer: W, directive: D, path: P) -> std::io::Result<()>
where
    W: Write,
    D: Display,
    P: Display,
{
    writeln!(
        writer,
        r#"{}{}"{}""#,
        terminals::COMMENT_START,
        directive,
        path
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(input: &str) -> Result<Config> {
        Config::read_from(input.as_bytes())
    }

    fn write(config: &Config) -> Result<String> {
        let mut buffer = Vec::new();
        config.write_to(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    fn round_trip(input: &str) -> Result<String> {
        write(&read(input)?)
    }

    /// A file in exactly the form RetroArch writes: references first, then entries sorted with `strcasecmp`, then includes.
    const CANONICAL: &str = r#"#reference "/Users/me/shared.cfg"
audio_driver = "coreaudio"
audiofilter_dir = "default"
input_player1_a = "k"
video_fullscreen = "true"
#include "/Users/me/base.cfg"
"#;

    // Whole-file behavior

    #[test]
    fn canonical_file_round_trips_byte_for_byte() -> Result<()> {
        let config = read(CANONICAL)?;
        let output = write(&config)?;
        assert_eq!(output, CANONICAL);
        // The emitted text must also read back as an equal Config
        assert_eq!(read(&output)?, config);
        Ok(())
    }

    #[test]
    fn output_normalized_to_libretro_ordering() -> Result<()> {
        // Same content as `CANONICAL`, but shuffled: an include first, a reference last, entries out of order. Note that `audio_driver` sorts before `audiofilter_dir` because `_` (0x5F) precedes `f` (0x66) once case is folded.
        let shuffled = r#"#include "/Users/me/base.cfg"
video_fullscreen = "true"
audiofilter_dir = "default"
input_player1_a = "k"
audio_driver = "coreaudio"
#reference "/Users/me/shared.cfg"
"#;
        assert_eq!(round_trip(shuffled)?, CANONICAL);
        Ok(())
    }

    #[test]
    fn empty_input_is_empty_config() -> Result<()> {
        let config = read("")?;
        assert_eq!(config, Config::default());
        assert_eq!(write(&config)?, "");
        Ok(())
    }

    #[test]
    fn replace_reports_whether_config_changed() -> Result<()> {
        let mut config = read(
            r#"video_fullscreen = "false"
"#,
        )?;

        // Existing key, different value: updated.
        assert!(config.replace("video_fullscreen", "true")?);
        // Same value again: not updated, which is what lets `configure` skip rewriting the file.
        assert!(!config.replace("video_fullscreen", "true")?);
        // New key: updated, and it lands in sorted position.
        assert!(config.replace("audio_driver", "coreaudio")?);

        assert_eq!(
            write(&config)?,
            r#"audio_driver = "coreaudio"
video_fullscreen = "true"
"#
        );
        Ok(())
    }

    // Lines that carry no entry

    #[test]
    fn blank_whitespace_only_and_comment_lines_are_ignored() -> Result<()> {
        let input = concat!(
            "# a comment\n",
            "\n",
            "   \n",
            "	\n",
            r#"  # an indented comment
video_fullscreen = "true"
#include
#reference
  #include "indented.cfg"
"#
        );
        assert_eq!(
            round_trip(input)?,
            r#"video_fullscreen = "true"
"#
        );
        Ok(())
    }

    #[test]
    fn lines_without_value_are_dropped() -> Result<()> {
        // No `=` at all, no whitespace between key and `=` (so the whole thing is the key), and a lone `=`.
        let input = r#"just_a_key
no_space="1"
= "orphan"
video_fullscreen = "true"
"#;
        assert_eq!(
            round_trip(input)?,
            r#"video_fullscreen = "true"
"#
        );
        Ok(())
    }

    #[test]
    fn form_feed_and_vertical_tab_not_considered_whitespace() -> Result<()> {
        // libretro's `ISSPACE` is only space, tab, CR and LF (unlike C's `isspace`), so `\f` and `\v` cannot separate a key from its `=`.
        let input = "a\x0b= \"1\"\n\x0cb = \"2\"\nc\t=\t\"3\"\n";
        assert_eq!(
            round_trip(input)?,
            r#"c = "3"
"#
        );
        Ok(())
    }

    // Values

    #[test]
    fn number_sign_inside_quoted_value_is_not_comment() -> Result<()> {
        let input = r#"savefile_directory = "/Users/me/a#b/c"
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    #[test]
    fn number_sign_after_value_starts_comment() -> Result<()> {
        let input = r#"quoted = "1" # note
unquoted = 2 # note
"#;
        assert_eq!(
            round_trip(input)?,
            r#"quoted = "1"
unquoted = "2"
"#
        );
        Ok(())
    }

    #[test]
    fn unquoted_values_end_at_whitespace_and_emitted_quoted() -> Result<()> {
        let input = r#"a = foo bar
b = "x" trailing text
"#;
        assert_eq!(
            round_trip(input)?,
            r#"a = "foo"
b = "x"
"#
        );
        Ok(())
    }

    #[test]
    fn empty_and_unterminated_values() -> Result<()> {
        let input = r#"empty = ""
unterminated = "runs to the end of the line
"#;
        assert_eq!(
            round_trip(input)?,
            r#"empty = ""
unterminated = "runs to the end of the line"
"#
        );
        Ok(())
    }

    #[test]
    fn crlf_line_endings_normalized() -> Result<()> {
        let input = "a = \"1\"\r\nb = 2\r\n#include \"x.cfg\"\r\n";
        assert_eq!(
            round_trip(input)?,
            r#"a = "1"
b = "2"
#include "x.cfg"
"#
        );
        Ok(())
    }

    // Keys

    #[test]
    fn duplicate_keys_keep_first_value() -> Result<()> {
        let input = r#"a = "first"
a = "second"
"#;
        assert_eq!(
            round_trip(input)?,
            r#"a = "first"
"#
        );
        Ok(())
    }

    #[test]
    fn keys_differing_only_by_case_are_distinct() -> Result<()> {
        // They compare equal when case is folded, so the tiebreak is plain byte order: `F` (0x46) before `f` (0x66).
        let input = r#"foo = "lower"
Foo = "upper"
"#;
        assert_eq!(
            round_trip(input)?,
            r#"Foo = "upper"
foo = "lower"
"#
        );
        Ok(())
    }

    // Directives

    #[test]
    fn directive_paths_emitted_quoted_in_original_order() -> Result<()> {
        let input = r#"#include b.cfg
#include "a file with spaces.cfg"
#reference z.cfg
#reference "another path.cfg"
"#;
        assert_eq!(
            round_trip(input)?,
            r#"#reference "z.cfg"
#reference "another path.cfg"
#include "b.cfg"
#include "a file with spaces.cfg"
"#
        );
        Ok(())
    }

    #[test]
    fn empty_reference_path_retained_empty_include_path_dropped() -> Result<()> {
        // libretro retains an empty reference path and drops an empty include path; we mirror both.
        let input = r#"#reference ""
#include ""
"#;
        assert_eq!(
            round_trip(input)?,
            r#"#reference ""
"#
        );
        Ok(())
    }
}
