//! Reader/writer for RetroArch's `retroarch.cfg`, following the grammar of
//! `libretro-common/file/config_file.c` at the RetroArch v1.22.2 tag.
//!
//! - Lines are split on `\n`. A line starting with `#` in column 0 is either a
//!   `#include "path"` / `#reference "path"` directive or a comment.
//! - Otherwise a line is `key = value`: the key is a run of ASCII graphic
//!   characters, followed by optional whitespace, `=`, and the value.
//! - A value is either `"quoted"` (ends at the next `"`; there is no escaping,
//!   and an unterminated quote runs to end of line) or unquoted (a run of ASCII
//!   graphic characters).
//! - A `#` after the start of the line begins a comment unless it falls inside
//!   the first quoted literal.
//! - The topmost entry wins when a key repeats. Lookups are case-sensitive.
//! - RetroArch writes `#reference` lines, then entries sorted with `strcasecmp`,
//!   then `#include` lines, each entry as `key = "value"`.
//!
//! Deliberate deviations: repeated keys collapse to the topmost entry, and
//! lines with an empty key or an empty `#reference` path are dropped.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs,
    io::{ErrorKind, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use tempfile::NamedTempFile;

/// A config key ordered like `strcasecmp` (ASCII case-insensitive), with a
/// byte-wise tiebreak so keys differing only by case stay distinct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Key(String);

impl Key {
    pub fn as_str(&self) -> &str {
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

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetroArchConfig {
    pub config: BTreeMap<Key, String>,
    pub includes: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum Line {
    Ignored,
    Include(String),
    Reference(String),
    Entry(String, String),
}

/// libretro's `ISSPACE` (a `lr_char_props` table lookup): space, tab, LF and CR
/// only. Unlike C `isspace`, it excludes `\f` and `\v`.
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// Port of `config_file_extract_value`.
fn extract_value(s: &str) -> String {
    let s = s.trim_start_matches(is_space);
    if let Some(rest) = s.strip_prefix('"') {
        // Up to the next quote, or to the end of the line if unterminated.
        rest.split('"').next().unwrap_or("").to_string()
    } else {
        s.chars().take_while(char::is_ascii_graphic).collect()
    }
}

/// Port of the non-directive path of `config_file_strip_comment`: cut at the
/// first `#` unless it lies inside the first quoted literal.
fn strip_comment(line: &str) -> &str {
    let Some(hash) = line.find('#') else {
        return line;
    };
    if let Some(open) = line.find('"') {
        if open < hash {
            if let Some(close_rel) = line[open + 1..].find('"') {
                if open + 1 + close_rel > hash {
                    return line;
                }
            }
        }
    }
    &line[..hash]
}

/// Port of `config_file_parse_line`.
fn parse_line(raw: &str) -> Line {
    if let Some(comment) = raw.strip_prefix('#') {
        let (is_include, rest) = if let Some(r) = comment.strip_prefix("include ") {
            (true, r)
        } else if let Some(r) = comment.strip_prefix("reference ") {
            (false, r)
        } else {
            return Line::Ignored;
        };
        if rest.is_empty() {
            return Line::Ignored;
        }
        let path = extract_value(rest);
        if path.is_empty() {
            return Line::Ignored;
        }
        return if is_include {
            Line::Include(path)
        } else {
            Line::Reference(path)
        };
    }
    let line = strip_comment(raw).trim_start_matches(is_space);
    let key: String = line.chars().take_while(char::is_ascii_graphic).collect();
    let rest = line[key.len()..].trim_start_matches(is_space);
    let Some(value) = rest.strip_prefix('=') else {
        return Line::Ignored; // no '=' after the key
    };
    if key.is_empty() {
        return Line::Ignored;
    }
    Line::Entry(key, extract_value(value))
}

/// Keys are ASCII graphic runs. `#` would start a comment, and `"` could make
/// a later `#` in the value look like it's outside the quoted literal.
fn check_key(key: &str) -> Result<()> {
    if key.is_empty()
        || !key.bytes().all(|b| b.is_ascii_graphic())
        || key.contains(['#', '"'])
    {
        bail!("invalid RetroArch config key: {key:?}");
    }
    Ok(())
}

/// There is no escaping, so values and paths can't hold `"` or a line break.
/// `#include` / `#reference` paths are written raw, like values.
fn check_text(what: &str, text: &str) -> Result<()> {
    if text.contains(['"', '\n']) {
        bail!("{what} can't be represented in retroarch.cfg: {text:?}");
    }
    Ok(())
}

impl RetroArchConfig {
    pub fn parse(text: &str) -> Self {
        let mut cfg = Self::default();
        for line in text.split('\n').filter(|l| !l.is_empty()) {
            match parse_line(line) {
                Line::Ignored => {}
                Line::Include(path) => cfg.includes.push(path),
                Line::Reference(path) => cfg.references.push(path),
                // Topmost entry wins (don't `collect()`: that keeps the last).
                Line::Entry(key, value) => {
                    cfg.config.entry(Key(key)).or_insert(value);
                }
            }
        }
        cfg
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.config.get(&Key(key.to_string())).map(String::as_str)
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        check_key(key)?;
        check_text(&format!("value for {key:?}"), value)?;
        self.config.insert(Key(key.to_string()), value.to_string());
        Ok(())
    }

    /// Validates everything first, because deserialized data never went
    /// through `set`.
    pub fn serialize(&self) -> Result<String> {
        let mut out = String::new();
        for r in &self.references {
            check_text("reference path", r)?;
            out.push_str(&format!("#reference \"{r}\"\n"));
        }
        for (k, v) in &self.config {
            check_key(k.as_str())?;
            check_text(&format!("value for {:?}", k.as_str()), v)?;
            out.push_str(&format!("{} = \"{v}\"\n", k.as_str()));
        }
        for i in &self.includes {
            check_text("include path", i)?;
            out.push_str(&format!("#include \"{i}\"\n"));
        }
        Ok(out)
    }
}

/// Merge `desired` into the RetroArch config at `path` and atomically replace
/// the file. A missing file is treated as empty. Does nothing if the result
/// is byte-identical to what's already there.
pub fn apply_settings(path: &Path, desired: &[(&str, &str)]) -> Result<()> {
    let existing = match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("Could not read {}", path.display())),
    };

    let mut cfg = existing
        .as_deref()
        .map(RetroArchConfig::parse)
        .unwrap_or_default();
    for (key, value) in desired {
        cfg.set(key, value)?;
    }
    let new_text = cfg.serialize()?;

    if existing.as_deref() == Some(new_text.as_str()) {
        return Ok(());
    }

    let dir = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(dir).with_context(|| format!("Could not create {}", dir.display()))?;

    // Temp file in the same directory so the rename stays on one filesystem.
    let mut tmp = NamedTempFile::new_in(dir).context("Could not create temporary file")?;
    tmp.write_all(new_text.as_bytes())
        .context("Could not write temporary file")?;
    tmp.as_file().sync_all().context("Could not sync temporary file")?;

    // NamedTempFile creates 0600; keep the original mode, or default to 0644.
    let perms = match fs::metadata(path) {
        Ok(meta) => meta.permissions(),
        Err(_) => fs::Permissions::from_mode(0o644),
    };
    fs::set_permissions(tmp.path(), perms).context("Could not set permissions")?;

    tmp.persist(path)
        .with_context(|| format!("Could not replace {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_unquoted_comments_and_topmost_wins() {
        let c = RetroArchConfig::parse(
            "# a comment\n\
             video_fullscreen = \"true\"\n\
             audio_latency = 64 # trailing\n\
             savefile_directory = \"/Users/x/a#b\"\n\
             empty = \"\"\n\
             bad=\"no space before equals\"\n\
             unterminated = \"to end of line\n\
             video_fullscreen = \"false\"\n",
        );
        assert_eq!(c.get("video_fullscreen"), Some("true"));
        assert_eq!(c.get("audio_latency"), Some("64"));
        assert_eq!(c.get("savefile_directory"), Some("/Users/x/a#b"));
        assert_eq!(c.get("empty"), Some(""));
        assert_eq!(c.get("bad"), None);
        assert_eq!(c.get("unterminated"), Some("to end of line"));
    }

    #[test]
    fn a_quote_ends_the_value_with_no_escaping() {
        let c = RetroArchConfig::parse("a = \"x\\\"y\"\nb = \"C:\\dir\\\"\n");
        assert_eq!(c.get("a"), Some("x\\")); // backslash is an ordinary byte
        assert_eq!(c.get("b"), Some("C:\\dir\\"));
    }

    #[test]
    fn directives_only_count_in_column_zero() {
        let c = RetroArchConfig::parse(
            "#include \"base.cfg\"\n  #include \"indented.cfg\"\n#reference \"/ref\"\na = \"1\"\n",
        );
        assert_eq!(c.includes, ["base.cfg"]);
        assert_eq!(c.references, ["/ref"]);
        assert_eq!(c.get("a"), Some("1"));
    }

    #[test]
    fn output_is_sorted_like_strcasecmp() {
        let mut c = RetroArchConfig::default();
        for k in ["Zeta", "audiofilter", "audio_driver", "B_key", "a_key"] {
            c.set(k, "1").unwrap();
        }
        let keys: Vec<_> = c.config.keys().map(Key::as_str).collect();
        // '_' (0x5F) sorts before lowercase letters; case is ignored.
        assert_eq!(keys, ["a_key", "audio_driver", "audiofilter", "B_key", "Zeta"]);
    }

    #[test]
    fn canonical_file_roundtrips_byte_for_byte() {
        let src = "#reference \"/ref\"\n\
                   audio_driver = \"coreaudio\"\n\
                   audiofilter_dir = \"default\"\n\
                   video_fullscreen = \"true\"\n\
                   #include \"base.cfg\"\n";
        assert_eq!(RetroArchConfig::parse(src).serialize().unwrap(), src);
    }

    #[test]
    fn unsorted_input_is_normalized() {
        let c = RetroArchConfig::parse("b = \"2\"\na = \"1\"\n#include \"x\"\nc = \"3\"\n");
        assert_eq!(c.serialize().unwrap(), "a = \"1\"\nb = \"2\"\nc = \"3\"\n#include \"x\"\n");
    }

    #[test]
    fn set_rejects_unrepresentable() {
        let mut c = RetroArchConfig::default();
        assert!(c.set("k", "has\"quote").is_err());
        assert!(c.set("k", "line\nbreak").is_err());
        assert!(c.set("bad key", "v").is_err());
        assert!(c.set("bad#key", "v").is_err());
        assert!(c.set("bad\"key", "v").is_err());
        assert!(c.set("", "v").is_err());
        assert!(c.set("a=b", "v").is_ok());
    }

    #[test]
    fn every_representable_value_roundtrips() {
        let alphabet = ['a', '\\', '#', ' ', '=', '\r'];
        let mut inputs = vec![String::new()];
        let mut layer = vec![String::new()];
        for _ in 0..6 {
            layer = layer
                .iter()
                .flat_map(|s| alphabet.iter().map(move |c| format!("{s}{c}")))
                .collect();
            inputs.extend(layer.iter().cloned());
        }
        for v in inputs {
            let mut c = RetroArchConfig::default();
            c.set("k", &v).unwrap();
            let text = c.serialize().unwrap();
            assert_eq!(RetroArchConfig::parse(&text).get("k"), Some(v.as_str()), "{v:?}");
        }
    }

    #[test]
    fn serde_derive_roundtrips_and_serialize_still_validates() {
        let c = RetroArchConfig::parse("b = \"2\"\na = \"1\"\n#include \"x\"\n");
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(
            json,
            r#"{"config":{"a":"1","b":"2"},"includes":["x"],"references":[]}"#
        );
        assert_eq!(serde_json::from_str::<RetroArchConfig>(&json).unwrap(), c);

        // Deserialization bypasses `set`, so serialize() must catch bad data.
        let bad: RetroArchConfig =
            serde_json::from_str(r#"{"config":{"k":"a\"b"},"includes":[],"references":[]}"#)
                .unwrap();
        assert!(bad.serialize().is_err());
    }

    #[test]
    fn apply_creates_updates_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config/retroarch.cfg");

        apply_settings(&path, &[("a", "1")]).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "a = \"1\"\n");
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o644);

        fs::write(&path, "other = \"x\"\na = \"1\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        apply_settings(&path, &[("a", "2"), ("b", "3")]).unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "a = \"2\"\nb = \"3\"\nother = \"x\"\n"
        );
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);

        let before = fs::metadata(&path).unwrap().modified().unwrap();
        apply_settings(&path, &[("a", "2"), ("b", "3")]).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), before);
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn form_feed_and_vertical_tab_are_not_whitespace() {
        // libretro's ISSPACE excludes \f and \v, so neither can separate tokens.
        let c = RetroArchConfig::parse("a\x0b= \"1\"\n\x0cb = \"2\"\nc\t=\t\"3\"\n");
        assert_eq!(c.get("a"), None);
        assert_eq!(c.get("b"), None);
        assert_eq!(c.get("c"), Some("3"));
    }
}
