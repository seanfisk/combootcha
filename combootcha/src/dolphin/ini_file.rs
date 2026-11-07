//! Parser and emitter for Dolphin's INI files (`Dolphin.ini`, `GFX.ini`, `GCPadNew.ini`, and friends), based on these sources from Dolphin 2609a (the git tag `2609a`):
//!
//! - https://github.com/dolphin-emu/dolphin/blob/2609a/Source/Core/Common/IniFile.h
//! - https://github.com/dolphin-emu/dolphin/blob/2609a/Source/Core/Common/IniFile.cpp
//! - https://github.com/dolphin-emu/dolphin/blob/2609a/Source/Core/Common/StringUtil.cpp (`StripWhitespace`, `StripQuotes`, and the case-insensitive comparisons)
//!
//! Comments copied from the Dolphin source code have been prefixed as such.
//!
//! Behavior mirrored from Dolphin:
//!
//! - Section names and keys are matched case-insensitively (ASCII only). The spelling that was seen first is kept.
//! - Sections and keys are written in the order they were added, not sorted.
//! - Repeating a section header merges into the first section of that name. Repeating a key keeps its original position but the *last* value wins.
//! - Whitespace around keys and values is stripped, as is one pair of quotes around a value. Values are written unquoted, as `key = value`. A `#` only starts a comment at the very beginning of a line.
//! - A line with no `=`, a comment, a whitespace-only line, or any line starting with `$`, `+` or `*` is a "raw line", kept verbatim and separately from the keys.
//! - A section is written as either its keys or its raw lines, never both: **raw lines are discarded from any section that has keys**. Sections with neither are not written.
//! - Lines before the first section, text after a section's `]`, and a `[` line with no `]` are dropped.
//!
//! Deliberate differences from Dolphin, following the policy of the RetroArch parser in this program:
//!
//! - Anything that could not be read back as the same thing is an error, both when reading and in `replace`: see `validate_section_name`, `validate_key` and `validate_value`. Dolphin silently normalizes (or mis-reads) these.
//! - Whatever Dolphin silently drops is dropped with a warning, including raw lines that are discarded on write.
//! - An entry with an empty key (`= value`) is dropped with a warning. Dolphin keeps it.

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info, trace, warn};
use std::borrow::Cow;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Config {
    // Sections are few and each is small, so lookups are linear scans. This also lets each section keep its original spelling without any normalization of the name.
    sections: Vec<Section>,
}

#[derive(Debug, PartialEq, Eq)]
struct Section {
    name: String,
    entries: Vec<Entry>,
    /// Dolphin: `m_lines`
    lines: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    key: String,
    value: String,
}

/// Where a value came from. This only affects how updates are logged.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// The caller of `replace`
    Caller,
    /// A line of the file being read
    File,
}

impl Section {
    fn new(name: String) -> Self {
        Self {
            name,
            entries: Vec::new(),
            lines: Vec::new(),
        }
    }
}

impl Config {
    // Generics are added to public functions, whereas the types in the private `set` function are concrete. This ensures that concrete copies generated for the generic remain small and is a common Rust pattern.

    /// Ensure `key` in `section` is set to `value`, returning whether the configuration was updated.
    pub(crate) fn replace<'a, S, K, V>(&mut self, section: S, key: K, value: V) -> Result<bool>
    where
        S: Into<Cow<'a, str>>,
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        self.set(section.into(), key.into(), value.into(), Source::Caller)
    }

    /// Dolphin: `IniFile::GetOrCreateSection`
    fn get_or_create_section(&mut self, name: Cow<'_, str>, source: Source) -> &mut Section {
        let index = match self
            .sections
            .iter()
            .position(|section| section.name.eq_ignore_ascii_case(&name))
        {
            Some(index) => index,
            None => {
                if source == Source::Caller {
                    info!("Adding new section {name:?}");
                } else {
                    debug!("Adding new section {name:?}");
                }
                self.sections.push(Section::new(name.into_owned()));
                self.sections.len() - 1
            }
        };
        &mut self.sections[index]
    }

    /// Set a key in a section, creating the section if necessary. Returns whether the configuration was updated.
    ///
    /// Dolphin: `IniFile::Section::Set`
    fn set(
        &mut self,
        section: Cow<'_, str>,
        key: Cow<'_, str>,
        value: Cow<'_, str>,
        source: Source,
    ) -> Result<bool> {
        // Design decision: validate everything before changing anything, so that a failure never leaves the config partially updated
        validate_section_name(&section)?;
        validate_key(&key)?;
        validate_value(&value)?;

        let section = self.get_or_create_section(section, source);
        let section_name = section.name.as_str();
        Ok(
            match section
                .entries
                .iter()
                .position(|entry| entry.key.eq_ignore_ascii_case(&key))
            {
                None => {
                    // If replacing, it helps to know that the key wasn't there already. Otherwise it can be somewhat misleading why the file is being written in the caller.
                    if source == Source::Caller {
                        info!("Adding new key {key:?} with value {value:?} to section {section_name:?}");
                    } else {
                        debug!("Adding new key {key:?} with value {value:?} to section {section_name:?}");
                    }
                    section.entries.push(Entry {
                        key: key.into_owned(),
                        value: value.into_owned(),
                    });
                    true
                }
                Some(index) => {
                    // Dolphin: `values.insert_or_assign` keeps the original key (and its position in `keys_order`) and replaces the value
                    let entry = &mut section.entries[index];
                    if entry.value == value {
                        debug!(
                            "New value {value:?} for key {:?} in section {section_name:?} matches existing value; no need to update",
                            entry.key
                        );
                        false
                    } else if source == Source::Caller {
                        info!(
                            "Replacing existing value {:?} for key {:?} in section {section_name:?} with desired value {value:?}",
                            entry.value, entry.key
                        );
                        entry.value = value.into_owned();
                        true
                    } else {
                        warn!(
                            "Duplicate key {key:?} in section {section_name:?}: value {value:?} overrides earlier value {:?}",
                            entry.value
                        );
                        entry.value = value.into_owned();
                        true
                    }
                }
            },
        )
    }

    /// Dolphin: `IniFile::Load`
    pub(crate) fn read_from<R: Read>(reader: R) -> Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut config = Self::default();
        // The name of the section most recently introduced by a header, as written in the file
        let mut current_section: Option<String> = None;
        let mut line = String::new();
        let mut line_number = 1;
        loop {
            let chars_read = reader
                .read_line(&mut line)
                .with_context(|| format!("Cannot read config line {line_number}"))?;
            if chars_read == 0 {
                break;
            }
            // Dolphin: `std::getline` does not include the delimiter
            if line.ends_with(terminals::NEWLINE) {
                line.pop();
            }
            let mut text = line.as_str();

            // Dolphin: Skips the UTF-8 BOM at the start of files. Notepad likes to add this.
            if line_number == 1 {
                text = text.strip_prefix(terminals::BOM).unwrap_or(text);
            }

            // Dolphin: Check for CRLF eol and convert it to LF
            // (Dolphin only does this when not building for Windows, which is the case we care about)
            text = text.strip_suffix(terminals::CARRIAGE_RETURN).unwrap_or(text);

            match parse_line(text) {
                LineContent::Ignored => {
                    trace!("Ignoring line {line_number}");
                    Ok(())
                }
                LineContent::Malformed(reason) => {
                    warn!("Ignoring malformed line {line_number}: {reason}");
                    Ok(())
                }
                LineContent::Section { name, trailing } => {
                    if !strip_whitespace(trailing).is_empty() {
                        warn!(
                            "Ignoring text {trailing:?} after the header of section {name:?} on line {line_number}"
                        );
                    }
                    validate_section_name(name).map(|()| {
                        // Dolphin: New section!
                        debug!("Line {line_number} starts section {name:?}");
                        config.get_or_create_section(Cow::Borrowed(name), Source::File);
                        current_section = Some(name.to_string());
                    })
                }
                LineContent::Raw(raw) => {
                    match &current_section {
                        Some(section) => {
                            trace!("Keeping raw line {line_number} verbatim in section {section:?}");
                            config
                                .get_or_create_section(Cow::Borrowed(section), Source::File)
                                .lines
                                .push(raw.to_string());
                        }
                        None => warn!("Ignoring line {line_number} before the first section"),
                    }
                    Ok(())
                }
                LineContent::Entry { key, value } => match &current_section {
                    Some(section) => config
                        .set(
                            Cow::Borrowed(section),
                            Cow::Borrowed(key),
                            Cow::Borrowed(value),
                            Source::File,
                        )
                        .map(|_| ()),
                    None => {
                        warn!("Ignoring line {line_number} before the first section");
                        Ok(())
                    }
                },
            }
            .with_context(|| format!("Cannot parse config line {line_number}"))?;
            line.clear();
            line_number += 1;
        }
        Ok(config)
    }

    /// Dolphin: `IniFile::Save`
    pub(crate) fn write_to<W: Write>(&self, writer: W) -> Result<()> {
        let mut writer = BufWriter::new(writer);
        for section in self.sections.iter() {
            if section.entries.is_empty() && section.lines.is_empty() {
                debug!("Not writing empty section {:?}", section.name);
                continue;
            }
            writeln!(writer, "[{}]", section.name)
                .with_context(|| format!("Cannot write header of section {:?}", section.name))?;

            if section.entries.is_empty() {
                for line in section.lines.iter() {
                    writeln!(writer, "{line}").with_context(|| {
                        format!("Cannot write raw line in section {:?}", section.name)
                    })?;
                }
            } else {
                // Dolphin writes a section's raw lines only if it has no keys. Mirror that, but not silently.
                let discarded = section
                    .lines
                    .iter()
                    .filter(|line| !strip_whitespace(line).is_empty())
                    .count();
                if discarded > 0 {
                    warn!(
                        "Discarding {discarded} raw line(s) from section {:?}: Dolphin only keeps a section's raw lines if it has no key-value entries",
                        section.name
                    );
                }
                for entry in section.entries.iter() {
                    writeln!(writer, "{} = {}", entry.key, entry.value).with_context(|| {
                        format!(
                            "Cannot write entry {:?} in section {:?}",
                            entry.key, section.name
                        )
                    })?;
                }
            }
        }
        writer.flush()?;
        Ok(())
    }
}

mod terminals {
    pub(super) const NEWLINE: char = '\n';
    pub(super) const CARRIAGE_RETURN: char = '\r';
    pub(super) const BOM: char = '\u{feff}';
    pub(super) const SECTION_START: char = '[';
    pub(super) const SECTION_END: char = ']';
    pub(super) const COMMENT_START: char = '#';
    pub(super) const EQUALS: char = '=';
    pub(super) const QUOTE: char = '"';
    /// Dolphin: Lines starting with these are kept verbatim
    pub(super) const RAW_LINE_PREFIXES: [char; 3] = ['$', '+', '*'];
}

/// Dolphin: `StripWhitespace` strips exactly these characters
fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

/// Port of Dolphin's `StripWhitespace`: "Turns "\n\r\t hello " into "hello" (trims at the start and end but not inside)."
fn strip_whitespace(s: &str) -> &str {
    s.trim_matches(is_whitespace)
}

// Port of Dolphin's `StripQuotes`.
// Dolphin: "\"hello\"" is turned to "hello"
// Dolphin: This one assumes that the string has already been space stripped in both
// Dolphin: ends, as done by StripWhitespace above, for example.
fn strip_quotes(s: &str) -> &str {
    // A lone quote character is both the first and the last character, so it is stripped to an empty string
    if s.starts_with(terminals::QUOTE) && s.ends_with(terminals::QUOTE) {
        s.get(1..s.len() - 1).unwrap_or("")
    } else {
        s
    }
}

/// Port of Dolphin's `IniFile::ParseLine`. Returns the key and the value, which are both empty if the line is not a `key = value` line.
fn parse_key_and_value(line: &str) -> (&str, &str) {
    if line.is_empty() || line.starts_with(terminals::COMMENT_START) {
        return ("", "");
    }
    match line.find(terminals::EQUALS) {
        Some(first_equals) => {
            // Dolphin: Yes, a valid line!
            (
                strip_whitespace(&line[..first_equals]),
                strip_quotes(strip_whitespace(&line[first_equals + 1..])),
            )
        }
        None => ("", ""),
    }
}

enum LineContent<'a> {
    Section { name: &'a str, trailing: &'a str },
    Raw(&'a str),
    Entry { key: &'a str, value: &'a str },
    Malformed(&'static str),
    Ignored,
}

/// Port of the line classification in Dolphin's `IniFile::Load`, which is entangled with the state of the section being read. This port classifies the line without mutating any state.
fn parse_line(line: &str) -> LineContent<'_> {
    if line.is_empty() {
        return LineContent::Ignored;
    }

    if let Some(after_bracket) = line.strip_prefix(terminals::SECTION_START) {
        return match after_bracket.find(terminals::SECTION_END) {
            Some(end) => LineContent::Section {
                name: &after_bracket[..end],
                trailing: &after_bracket[end + 1..],
            },
            // Dolphin does not even start a new section here, so the lines that follow still belong to the previous one
            None => LineContent::Malformed("section header without a closing bracket"),
        };
    }

    let (key, value) = parse_key_and_value(line);

    // Dolphin: Lines starting with '$', '*' or '+' are kept verbatim.
    // Kind of a hack, but the support for raw lines inside an INI is a hack anyway.
    if (key.is_empty() && value.is_empty()) || line.starts_with(terminals::RAW_LINE_PREFIXES) {
        LineContent::Raw(line)
    } else if key.is_empty() {
        // Dolphin keeps an entry with an empty key, but there is no good reason to want one
        LineContent::Malformed("entry with an empty key")
    } else {
        LineContent::Entry { key, value }
    }
}

// Everything below checks that text survives being written and read back as the same thing

/// A section header ends at the first `]`, and the header is a single line
fn validate_section_name(name: &str) -> Result<()> {
    trace!("Validating Dolphin INI section name {name:?}");
    if name.contains([terminals::SECTION_END, terminals::NEWLINE, terminals::CARRIAGE_RETURN]) {
        Err(anyhow!(
            "Invalid Dolphin INI section name {name:?}: it cannot contain `]` or a line break"
        ))
    } else {
        Ok(())
    }
}

fn validate_key(key: &str) -> Result<()> {
    trace!("Validating Dolphin INI key {key:?}");
    if key.is_empty() {
        bail!("Invalid Dolphin INI key {key:?}: it cannot be empty");
    }
    if key.contains([terminals::EQUALS, terminals::NEWLINE, terminals::CARRIAGE_RETURN]) {
        bail!("Invalid Dolphin INI key {key:?}: it cannot contain `=` or a line break");
    }
    if key != strip_whitespace(key) {
        bail!("Invalid Dolphin INI key {key:?}: whitespace around it is stripped when reading");
    }
    // A line starting with one of these is read as a section header, a comment, or a raw line, and not as an entry
    if key.starts_with([terminals::SECTION_START, terminals::COMMENT_START])
        || key.starts_with(terminals::RAW_LINE_PREFIXES)
    {
        bail!("Invalid Dolphin INI key {key:?}: it would be read back as something other than an entry");
    }
    Ok(())
}

fn validate_value(value: &str) -> Result<()> {
    trace!("Validating Dolphin INI value {value:?}");
    if value.contains([terminals::NEWLINE, terminals::CARRIAGE_RETURN]) {
        bail!("Invalid Dolphin INI value {value:?}: it cannot contain a line break");
    }
    if value != strip_whitespace(value) {
        bail!("Invalid Dolphin INI value {value:?}: whitespace around it is stripped when reading");
    }
    // Values are written unquoted, and a pair of quotes around the value is stripped when reading
    if value.starts_with(terminals::QUOTE) && value.ends_with(terminals::QUOTE) {
        bail!("Invalid Dolphin INI value {value:?}: quotes around it are stripped when reading");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test harness. Everything goes through `Config`'s read/write interface using in-memory byte buffers, so no test touches the filesystem. Raw strings are deliberately flush left: their contents are exactly the bytes of the file. Strings with trailing whitespace, carriage returns, or a BOM are ordinary strings with escapes, because those characters are invisible (and editors strip trailing whitespace).

    /// Parse `input` as a Dolphin INI file.
    fn read(input: &str) -> Result<Config> {
        Config::read_from(input.as_bytes())
    }

    /// Emit `config` as the text of a Dolphin INI file.
    fn write(config: &Config) -> Result<String> {
        let mut buffer = Vec::new();
        config.write_to(&mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Parse `input` and emit the result: what Combootcha does to a file it leaves otherwise unchanged.
    fn round_trip(input: &str) -> Result<String> {
        write(&read(input)?)
    }

    /// Look up a value, case-insensitively.
    fn get<'a>(config: &'a Config, section: &str, key: &str) -> Option<&'a str> {
        config
            .sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(section))?
            .entries
            .iter()
            .find(|e| e.key.eq_ignore_ascii_case(key))
            .map(|e| e.value.as_str())
    }

    /// A file in the form Dolphin writes.
    const CANONICAL: &str = r#"[Core]
CPUThread = True
GFXBackend = Metal
[Display]
Fullscreen = True
[Interface]
ConfirmStop = False
"#;

    // Whole-file behavior

    #[test]
    fn canonical_file_round_trips_byte_for_byte() -> Result<()> {
        let output = round_trip(CANONICAL)?;
        assert_eq!(output, CANONICAL);
        // The emitted text must also read back as an equal `Config`.
        assert_eq!(read(&output)?, read(CANONICAL)?);
        Ok(())
    }

    #[test]
    fn empty_input_is_an_empty_config() -> Result<()> {
        let config = read("")?;
        assert_eq!(config, Config::default());
        assert_eq!(write(&config)?, "");
        Ok(())
    }

    #[test]
    fn order_is_preserved_and_not_sorted() -> Result<()> {
        let input = r#"[Zebra]
z = 1
a = 2
[Apple]
b = 1
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    // replace

    #[test]
    fn replace_reports_whether_the_config_changed() -> Result<()> {
        let mut config = read(
            r#"[Core]
CPUThread = False
"#,
        )?;

        // Existing key, different value: updated.
        assert!(config.replace("Core", "CPUThread", "True")?);
        // Same value again: not updated, which is what lets the caller skip rewriting the file.
        assert!(!config.replace("Core", "CPUThread", "True")?);
        // New key in an existing section: updated, and appended to the section.
        assert!(config.replace("Core", "GFXBackend", "Metal")?);
        // New section: updated, and appended after the existing ones.
        assert!(config.replace("Display", "Fullscreen", "True")?);

        assert_eq!(
            write(&config)?,
            r#"[Core]
CPUThread = True
GFXBackend = Metal
[Display]
Fullscreen = True
"#
        );
        Ok(())
    }

    #[test]
    fn names_match_case_insensitively_and_keep_their_first_spelling() -> Result<()> {
        let mut config = read(
            r#"[Core]
CPUThread = False
"#,
        )?;
        assert!(config.replace("CORE", "cputhread", "True")?);
        assert!(!config.replace("core", "CPUTHREAD", "True")?);
        assert_eq!(
            write(&config)?,
            r#"[Core]
CPUThread = True
"#
        );
        Ok(())
    }

    // Sections and keys

    #[test]
    fn repeated_section_headers_merge_into_the_first() -> Result<()> {
        let input = r#"[A]
a = 1
[B]
b = 1
[a]
c = 1
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[A]
a = 1
c = 1
[B]
b = 1
"#
        );
        Ok(())
    }

    #[test]
    fn repeated_keys_keep_the_first_position_and_spelling_but_the_last_value() -> Result<()> {
        let input = r#"[S]
Key = 1
other = 2
key = 3
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[S]
Key = 3
other = 2
"#
        );
        Ok(())
    }

    #[test]
    fn section_names_keep_their_whitespace_and_can_be_empty() -> Result<()> {
        let input = r#"[ Core ]
a = 1
[]
b = 2
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    #[test]
    fn empty_sections_are_not_written() -> Result<()> {
        let input = r#"[Empty]
[Full]
k = v
[AlsoEmpty]
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[Full]
k = v
"#
        );
        Ok(())
    }

    // Keys and values

    #[test]
    fn whitespace_and_one_pair_of_quotes_are_stripped() -> Result<()> {
        let input = "[S]\n  key  =  \"quoted value\"  \n\tk2\t=\tplain\t\nk3 =\nk4 = \"\"\nk5 = \"\n";
        // An empty value is written as `key = ` with a trailing space, just as Dolphin does
        assert_eq!(
            round_trip(input)?,
            "[S]\nkey = quoted value\nk2 = plain\nk3 = \nk4 = \nk5 = \n"
        );
        Ok(())
    }

    #[test]
    fn only_the_first_equals_sign_separates_key_from_value() -> Result<()> {
        let input = r#"[S]
a = b = c
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    #[test]
    fn hash_only_starts_a_comment_at_the_beginning_of_a_line() -> Result<()> {
        let input = r#"[S]
a = 1 # this is part of the value
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    // Raw lines

    #[test]
    fn raw_lines_are_kept_verbatim_in_sections_without_entries() -> Result<()> {
        // A `$`, `+` or `*` line is raw even if it has an `=`, and so is a comment or a line without one.
        let input = r#"[OnFrame]
$Widescreen Hack
*0x80000000:dword:0x00000001
+0x80000004:dword:0x00000002
$Cheat = on
# a comment
just some text
"#;
        assert_eq!(round_trip(input)?, input);
        Ok(())
    }

    #[test]
    fn raw_lines_are_discarded_from_sections_with_entries() -> Result<()> {
        let input = r#"[S]
# comment
$cheat
key = 1
+patch
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[S]
key = 1
"#
        );
        Ok(())
    }

    #[test]
    fn replace_turns_a_raw_line_section_into_an_entry_section() -> Result<()> {
        let mut config = read(
            r#"[S]
$cheat
"#,
        )?;
        assert!(config.replace("S", "key", "1")?);
        assert_eq!(
            write(&config)?,
            r#"[S]
key = 1
"#
        );
        Ok(())
    }

    #[test]
    fn whitespace_only_lines_are_raw_lines_but_empty_lines_are_skipped() -> Result<()> {
        let input = "[S]\n\n   \n# c\n";
        assert_eq!(round_trip(input)?, "[S]\n   \n# c\n");
        Ok(())
    }

    #[test]
    fn a_lone_equals_sign_is_a_raw_line_but_an_entry_without_a_key_is_dropped() -> Result<()> {
        let input = r#"[S]
= orphan
k = v
[R]
=
= ""
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[S]
k = v
[R]
=
= ""
"#
        );
        Ok(())
    }

    // Lines Dolphin drops

    #[test]
    fn lines_before_the_first_section_are_dropped() -> Result<()> {
        let input = r#"# header
k = v
[S]
a = 1
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[S]
a = 1
"#
        );
        Ok(())
    }

    #[test]
    fn entries_before_the_first_section_are_dropped_without_being_validated() -> Result<()> {
        // `[x] = y` as a key would be an error inside a section, but here it is simply dropped
        let input = r#"  [x] = y
[S]
k = 1
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[S]
k = 1
"#
        );
        Ok(())
    }

    #[test]
    fn an_unterminated_header_is_ignored_and_does_not_change_the_section() -> Result<()> {
        let input = r#"[A]
a = 1
[B
b = 2
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[A]
a = 1
b = 2
"#
        );
        Ok(())
    }

    #[test]
    fn text_after_a_section_header_is_dropped() -> Result<()> {
        let input = r#"[A] trailing text
a = 1
[x] = y
b = 2
"#;
        assert_eq!(
            round_trip(input)?,
            r#"[A]
a = 1
[x]
b = 2
"#
        );
        Ok(())
    }

    // Line endings and encoding

    #[test]
    fn bom_and_crlf_are_removed_and_a_final_newline_is_optional() -> Result<()> {
        assert_eq!(round_trip("\u{feff}[S]\r\na = 1\r\nb = 2")?, "[S]\na = 1\nb = 2\n");
        Ok(())
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        assert!(Config::read_from(&b"[S]\nk = \xff\n"[..]).is_err());
    }

    // Input that cannot round-trip is an error, not silently altered

    #[test]
    fn input_that_cannot_round_trip_is_rejected() {
        // The first six are entries Dolphin reads (because their line does not *begin* with a special character) but whose keys would be read back as a header, a comment, or a raw line. The next is a value whose quotes would be stripped on the next read. The last two have a carriage return in the middle of a name or value, which cannot be written back without changing meaning.
        let inputs = [
            "[S]\n  [x] = y\n",
            "[S]\n  # c = d\n",
            "[S]\n  $a = b\n",
            "[S]\n  +a = b\n",
            "[S]\n  *a = b\n",
            "[S]\n  [x]y = z\n",
            "[S]\nk = \"\"x\"\"\n",
            "[a\rb]\nk = v\n",
            "[S]\nk = a\rb\n",
        ];
        for input in inputs {
            assert!(read(input).is_err(), "should have rejected {input:?}");
        }
    }

    #[test]
    fn read_errors_name_the_offending_line() {
        let input = r#"[S]
a = 1
  $b = 2
"#;
        let error = format!("{:#}", read(input).unwrap_err());
        assert!(error.contains("line 3"), "unexpected error: {error}");
    }

    #[test]
    fn replace_rejects_invalid_names_and_values_and_changes_nothing() {
        let mut config = Config::default();
        let invalid = [
            ("a]b", "k", "v"),
            ("a\nb", "k", "v"),
            ("a\rb", "k", "v"),
            ("S", "", "v"),
            ("S", "a=b", "v"),
            ("S", " k", "v"),
            ("S", "k ", "v"),
            ("S", "[k", "v"),
            ("S", "#k", "v"),
            ("S", "$k", "v"),
            ("S", "+k", "v"),
            ("S", "*k", "v"),
            ("S", "a\nb", "v"),
            ("S", "k", " v"),
            ("S", "k", "v "),
            ("S", "k", "\"v\""),
            ("S", "k", "\""),
            ("S", "k", "a\nb"),
            ("S", "k", "a\rb"),
        ];
        for (section, key, value) in invalid {
            assert!(
                config.replace(section, key, value).is_err(),
                "should have rejected [{section:?}] {key:?} = {value:?}"
            );
        }
        // Not even the section was created
        assert_eq!(config, Config::default());
    }

    #[test]
    fn replace_accepts_awkward_but_representable_text() -> Result<()> {
        let mut config = Config::default();
        assert!(config.replace("Section With Spaces", "Main Stick/Dead Zone", "`Click 1` | `Ctrl`")?);
        assert!(config.replace("S", "k", "\"half quoted")?);
        assert!(config.replace("S", "k2", "a = b # c [d] $e")?);
        let output = write(&config)?;
        assert_eq!(output, round_trip(&output)?);
        Ok(())
    }

    // Property: anything `replace` accepts survives a write and a read

    /// Every string of up to `max_length` characters from `alphabet`
    fn strings_from(alphabet: &[char], max_length: usize) -> Vec<String> {
        let mut all = vec![String::new()];
        let mut layer = all.clone();
        for _ in 0..max_length {
            layer = layer
                .iter()
                .flat_map(|prefix| alphabet.iter().map(move |c| format!("{prefix}{c}")))
                .collect();
            all.extend(layer.iter().cloned());
        }
        all
    }

    /// Replace one (section, key, value) in an empty config; if that is accepted, write and read it back and check that all three came back unchanged. Returns whether it was accepted.
    fn survives(section: &str, key: &str, value: &str) -> Result<bool> {
        let mut config = Config::default();
        if config.replace(section, key, value).is_err() {
            return Ok(false);
        }
        let reread = read(&write(&config)?)?;
        assert_eq!(
            get(&reread, section, key),
            Some(value),
            "[{section:?}] {key:?} = {value:?} did not survive"
        );
        assert_eq!(reread.sections[0].name, section);
        Ok(true)
    }

    #[test]
    fn every_accepted_value_key_and_section_name_survives_a_round_trip() -> Result<()> {
        // An alphabet chosen to be awkward for this grammar: whitespace, `=`, the characters that begin a header, comment, or raw line, quotes, and a multi-byte character
        let alphabet = ['a', ' ', '=', '"', '#', '[', ']', '$', '+', '*', 'é'];
        let (mut accepted, mut rejected) = (0, 0);
        for text in strings_from(&alphabet, 3) {
            for (section, key, value) in [("S", "k", text.as_str()), ("S", text.as_str(), "v"), (text.as_str(), "k", "v")] {
                if survives(section, key, value)? {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
            }
        }
        // Guard against a vacuous test
        assert!(accepted > 1000 && rejected > 1000, "{accepted} accepted, {rejected} rejected");
        Ok(())
    }
}
