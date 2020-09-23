use crate::verbose_command;
use anyhow::Result;
use nix::unistd::{chown, Uid};
use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use users::User;

pub fn install_formulae(standard_user: User) -> Result<()> {
    let brewfile_bytes = include_bytes!("Brewfile");
    // Write the Brewfile to an easy-to-access location so that manual commands can be run against it.
    let brewfile_dest_str = "/usr/local/Brewfile";
    let brewfile_dest = Path::new(brewfile_dest_str);

    let mut brewfile = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o400)
        .open(brewfile_dest)?;
    brewfile.write_all(brewfile_bytes)?;

    chown(
        brewfile_dest,
        Some(Uid::from_raw(standard_user.uid())),
        None,
    )?;

    verbose_command::run(
        Command::new("brew")
            .arg("bundle")
            .arg("install")
            .args(vec!["--file", brewfile_dest_str])
            .arg("--verbose")
            .arg("--no-lock") // Don't output Brewfile.lock.json
            .uid(standard_user.uid()),
    )
}
