mod config_file;

use anyhow::{Context, Result};
use log::{debug, info};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;
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
        let (mut config, original_permissions) = match File::open(&config_path) {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    debug!(
                        "Missing file {:?}; returning empty configuration",
                        config_path
                    );
                    Ok((Config::default(), std::fs::Permissions::from_mode(0o644)))
                } else {
                    Err(e).context("TODO")
                }
            }
            Ok(file) => {
                let config = Config::read_from(&file)?;
                let permissions = file.metadata()?.permissions();
                Ok((config, permissions))
            }
        }?;
        let mut updated = false;
        for (key, value) in ENTRIES {
            updated = updated || config.replace(*key, *value)?;
        }
        if updated {
            info!("Config updated; writing out"); // TODO Improve

            // Temp file in the same directory so the rename stays on one filesystem.
            let file = NamedTempFile::new_in(&config_dir)
                .with_context(|| format!("Could not create temporary file in {:?}", config_dir))?;
            config.write_to(&file)?;
            file.as_file()
                .sync_all()
                .context("Could not sync temporary file")?;

            debug!("Moving output file into original location");
            let file = file.persist(&config_path).with_context(|| {
                format!(
                    "Could not move temporary file back to original file's location at {:?}",
                    config_path
                )
            })?;

            debug!("Setting output file permissions");
            file.set_permissions(original_permissions)
                .with_context(|| {
                    format!(
                        "Error writing permissions to newly-written output file {:?}",
                        config_path
                    )
                })?;
        } else {
            info!("Nothing to update in config"); // TODO improve
        }
        Ok(())
    })
}
