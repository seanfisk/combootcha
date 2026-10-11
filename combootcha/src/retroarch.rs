use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};
use users::User;
use anyhow::{bail, Result};
use indexmap::IndexMap;

use scoby::UserExt;

const _SETTINGS: &[(&str, &str)] = &[("video_fullscreen", "true"), ("menu_driver", "ozone")];

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    // RetroArch uses a bespoke configuration format. Wonderful. Find the details here:
    //
    // https://github.com/libretro/libretro-common/blob/master/file/config_file.c
    // https://github.com/libretro/libretro-common/blob/master/include/file/config_file.h
    //
    // Upon start, RetroArch reads retroarch.cfg into memory and immediately writes it back. If retroarch.cfg is missing, empty, or missing some keys, the missing keys will get default values and be written back.
    // Upon quit, RetroArch always writes retroarch.cfg from memory with no regard to any changes that were made to the file externally while RetroArch was running.
    // When settings are changed, RetroArch stores them in memory but does not write them back to retroarch.cfg until quit. This means that the application must be quit cleanly for changes to be written.
    standard_user.as_effective_user(|| Ok(()))
}

/// Ordered like `strcasecmp` used by libretro, with a byte-wise tiebreak so
/// keys differing only by case stay distinct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Key(String);

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
pub struct Config {
    /// Ordered key/value entries (first occurrence wins, like RetroArch).
    pub entries: BTreeMap<Key, String>,
    /// `#reference "..."` lines. Written first.
    pub references: Vec<String>,
    /// `#include "..."` lines. Written last, like libretro does, so the
    /// file's own entries take precedence over included ones.
    pub includes: Vec<String>,
}

/// A port of `config_file_extract_value`.
fn extract_value(s: &str) -> String {
    let s = s.trim_start_matches([' ', '\t', '\r', '\n']);
    if let Some(rest) = s.strip_prefix('"') {
        // Unterminated literal: everything to end of line is the value.
        rest.split('"').next().unwrap_or("").to_string()
    } else {
        s.split(char::is_whitespace).next().unwrap_or("").to_string()
    }
}

/// Strip a trailing comment, ignoring `#` inside a quoted literal.
fn strip_comment(line: &str) -> &str {
    if let Some(hash) = line.find('#') {
        if let Some(open) = line.find('"') {
            if open < hash {
                if let Some(close_rel) = line[open + 1..].find('"') {
                    if open + 1 + close_rel > hash {
                        return line; // '#' is inside the literal
                    }
                }
            }
        }
        return &line[..hash];
    }
    line
}

impl RetroArchConfig {
    pub fn parse(text: &str) -> Self {
        let mut cfg = Self::default();
        for raw in text.lines() {
            let line = raw.trim_start_matches([' ', '\t']);
            if let Some(comment) = line.strip_prefix('#') {
                if let Some(p) = comment.strip_prefix("include ") {
                    let p = extract_value(p);
                    if !p.is_empty() {
                        cfg.includes.push(p);
                    }
                } else if let Some(p) = comment.strip_prefix("reference ") {
                    let p = extract_value(p);
                    if !p.is_empty() {
                        cfg.references.push(p);
                    }
                }
                continue; // all other comments are dropped
            }
            let line = strip_comment(line);
            let key_len = line.find(char::is_whitespace).unwrap_or(line.len());
            let (key, rest) = line.split_at(key_len);
            if key.is_empty() {
                continue;
            }
            let rest = rest.trim_start_matches([' ', '\t', '\r', '\n']);
            let Some(value) = rest.strip_prefix('=') else {
                continue; // no '=' after the key: invalid line, ignored
            };
            // Topmost entry wins, matching RetroArch's lookup.
            cfg.entries
                .entry(key.to_string())
                .or_insert_with(|| extract_value(value));
        }
        cfg
    }

    /// Update in place or append. The format has no escaping, so values can't
    /// contain `"` or newlines.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        if key.is_empty() || key.contains(char::is_whitespace) || key.contains('#') {
            bail!("invalid RetroArch config key: {key:?}");
        }
        if value.contains(['"', '\n', '\r']) {
            bail!("value for {key:?} can't be represented in retroarch.cfg: {value:?}");
        }
        self.entries.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn serialize(&self) -> String {
        let mut out = String::new();
        for r in &self.references {
            out.push_str(&format!("#reference \"{r}\"\n"));
        }
        for (k, v) in &self.entries {
            out.push_str(&format!("{k} = \"{v}\"\n"));
        }
        for i in &self.includes {
            out.push_str(&format!("#include \"{i}\"\n"));
        }
        out
    }
}
