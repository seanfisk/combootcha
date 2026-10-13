mod config_file;

use anyhow::{Context, Result};
use log::debug;
use std::fs::File;
use users::{os::unix::UserExt, User};

use config_file::Config;

use scoby::UserExt as OtherUserExt;

// TODO Check consistency of strcasecmp

const ENTRIES: &[(&str, &str)] = &[("video_fullscreen", "true"), ("menu_driver", "ozone")];

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    // RetroArch uses a bespoke configuration format. Wonderful. Find the details here:
    //
    // https://github.com/libretro/libretro-common/blob/master/file/config_file.c
    // https://github.com/libretro/libretro-common/blob/master/include/file/config_file.h
    //
    // Upon start, RetroArch reads retroarch.cfg into memory and immediately writes it back. If retroarch.cfg is missing, empty, or missing some keys, the missing keys will get default values and be written back.
    // Upon quit, RetroArch always writes retroarch.cfg from memory with no regard to any changes that were made to the file externally while RetroArch was running.
    // When settings are changed, RetroArch stores them in memory but does not write them back to retroarch.cfg until quit. This means that the application must be quit cleanly for changes to be written.
    standard_user.as_effective_user(|| {
        let config_dir = standard_user
            .home_dir()
            .join("Library/Application Support/RetroArch/config");
        let config_path = config_dir.join("retroarch.cfg");
        let mut config = match File::open(&config_path) {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    debug!(
                        "Missing file {:?}; returning empty configuration",
                        config_path
                    );
                    Ok(Config::default())
                } else {
                    Err(e).context("TODO")
                }
            }
            Ok(file) => Config::read_from(file),
        }?;
        let mut modified = false;
        for (key, value) in ENTRIES {
            modified = modified || config.replace(*key, *value)?;
        }
        Ok(())
    })
}
