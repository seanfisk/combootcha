use anyhow::Result;
use log::info;
use nix::unistd::{chown, Uid};

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use users::{os::unix::UserExt, User};

use crate::verbose_command;

pub(crate) fn install_system(standard_user: &User) -> Result<()> {
    info!("Considering Homebrew installation");

    if Path::new("/usr/local/bin/brew").exists() {
        info!("Hombrew is already installed");
        Ok(())
    } else {
        info!("Installing Homebrew…");
        // Yeah, we could pull this down with reqwest, but it's a bit simpler to use the exact command that Hombrew provides
        verbose_command::run(
            Command::new("/bin/bash")
                .arg("-c")
                .arg("$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)")
                .uid(standard_user.uid())
        ).map(|_| {
            info!("Homebrew installed successfully");
            ()
        })
    }
}

pub(crate) fn install_deps(standard_user: &User) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");

    let brewfile_bytes = include_bytes!("Brewfile");
    let brewfile_dest = standard_user.home_dir().join(".Brewfile");

    let mut brewfile = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o400)
        .open(&brewfile_dest)?;
    brewfile.write_all(brewfile_bytes)?;

    chown(
        &brewfile_dest,
        Some(Uid::from_raw(standard_user.uid())),
        None,
    )?;

    verbose_command::run(
        Command::new("brew")
            .arg("bundle")
            .arg("install")
            .arg("--verbose")
            .arg("--global")
            .uid(standard_user.uid()),
    )
}
