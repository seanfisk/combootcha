mod ini_file;

use anyhow::{Context, Result};
use log::{debug, info};
use std::fmt::Debug;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::NamedTempFile;
use users::{os::unix::UserExt, User};

use config_file::Config;

use scoby::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    // Upon start, RetroArch reads retroarch.cfg into memory. If retroarch.cfg is missing, empty, or missing some keys, the missing keys will get default values and immediately be written back.
    //
    // Upon quit, RetroArch always writes retroarch.cfg from memory with no regard to any changes that were made to the file externally while RetroArch was running.
    //
    // When settings are changed within RetroArch, the application stores them in memory but does not write them back to retroarch.cfg until quit. This means that the application must be quit cleanly for changes to be written.
    standard_user.as_effective_user(|| {
        let config_dir = standard_user
            .home_dir()
            .join("Library/Application Support/RetroArch/config");

        const SNES_EMULATOR: &str = "Snes9x";

        // TODO Consider renaming these variables not specific to the games since they're really not

        let alttpr_rmp_dir = config_dir.join("remaps").join(SNES_EMULATOR);
        let alttpr_cfg_dir = config_dir.join(SNES_EMULATOR);
        let alttpr_dir_name = Path::new("alttpr"); // This can also be done with macros, but whatever

        let fire_emblem_cfg_dir = config_dir.join("mGBA");

        for dir in [&alttpr_rmp_dir, &alttpr_cfg_dir, &fire_emblem_cfg_dir] {
            scoby::fs::ensure_dir(dir)?;
        }

        ensure_config(
            config_dir,
            "retroarch.cfg",
            &[],
        )?;

        ensure_config(
            alttpr_rmp_dir,
            alttpr_dir_name.with_extension("rmp"),
            &[],
        )?;

        ensure_config(
            fire_emblem_cfg_dir,
            "fire-emblem.cfg",
            &[
                // Fast forward doesn't have too many uses in these games, but here they are:
                //
                // 1. Loading state to an earlier turn, making some changes, then skipping through an Enemy Phase because you generally know what happens
                // 2. Arena
                //
                // The goal is to be slow enough to generally see what's going on, but fast enough to get through the gameplay quickly.
                ("fastforward_ratio", "4.000000"),
            ],
        )
    })
}

// TODO Learn how to genericize the entries?
/// Pass the directory because we will always have it
fn ensure_config<D, N>(dir: D, file_name: N, entries: &[(&str, &str)]) -> Result<()>
where
    D: AsRef<Path> + Debug,
    N: AsRef<Path> + Debug,
{
    let dir = dir.as_ref();
    let path = dir.join(file_name);
    info!("Reading RetroArch config from {:?}", path);
    let (mut config, original_permissions) = || -> Result<_> {
        match File::open(&path) {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    debug!("Config file {path:?} does not exist; starting with empty config");
                    Ok((Config::default(), std::fs::Permissions::from_mode(0o644)))
                } else {
                    Err(e.into())
                }
            }
            Ok(file) => {
                let config = Config::read_from(&file)?;
                let permissions = file.metadata()?.permissions();
                Ok((config, permissions))
            }
        }
    }()
    .with_context(|| format!("Could not read RetroArch config from {path:?}"))?;
    let mut updated = false;
    for &(key, value) in entries {
        updated |= config.replace(key, value)?;
    }
    if updated {
        info!("Writing updated RetroArch config to {path:?}");

        || -> Result<()> {
            // Create temp file in same directory as original file. This guarantees the files will be on the same filesystem, so the rename will be atomic.
            let temp_file = NamedTempFile::new_in(dir)
                .with_context(|| format!("Could not create temporary file in {:?}", dir))?;
            let temp_file_path = temp_file.path();
            debug!("Writing new RetroArch config to temp file {temp_file_path:?}");
            config.write_to(&temp_file)?;
            temp_file.as_file().sync_all().with_context(|| {
                format!("Could not sync temporary file at {:?}", temp_file_path)
            })?;

            debug!("Applying original permissions to {temp_file_path:?}");
            temp_file
                .as_file()
                .set_permissions(original_permissions)
                .with_context(|| format!("Error applying original permissions to {path:?}"))?;

            debug!("Moving temp file {temp_file_path:?} to {path:?}");
            let temp_file_path = temp_file_path.to_owned(); // Rust variable `temp_file` is moved when we call `persist` (I am not talking about being moved on the filesystem)
            temp_file.persist(&path).with_context(|| {
                format!("Could not move temporary file {temp_file_path:?} to {path:?}")
            })?;

            Ok(())
        }()
        .with_context(|| format!("Could not write RetroArch config to {path:?}"))?;
    } else {
        info!(
            "RetroArch config {:?} already populated correctly; not updating file",
            path
        );
    }
    Ok(())
}
