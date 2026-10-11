use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};
use users::User;

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
    pub config: BTreeMap<Key, String>,
    pub includes: Vec<String>,
    pub references: Vec<String>,
}
