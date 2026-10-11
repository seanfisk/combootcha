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

// Rust parser and emitter based on these sources:
//
// https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/include/file/config_file.h
// https://github.com/libretro/RetroArch/blob/v1.22.2/libretro-common/file/config_file.c
//
// The grammar has been updated in this pull: https://github.com/libretro/RetroArch/pull/19218
//
// This will very likely be included in a future release, but for now, we will target the grammar in version 1.22.2.
