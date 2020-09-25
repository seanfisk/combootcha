use anyhow::Result;
use log::info;
use users::User;

// use std::ffi::OsStr;
// use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

use crate::verbose_command;

pub(crate) fn set(standard_user: &User) -> Result<()> {
    info!("Querying Homebrew bin directory");
    let brew_prefix_output = verbose_command::run_output(
        Command::new("brew")
            .arg("--prefix")
            .uid(standard_user.uid()),
    )?;
    let brew_prefix = Path::new(std::str::from_utf8(&brew_prefix_output)?.trim_end_matches('\n'));
    let brew_bin = brew_prefix.join("bin");

    info!("Homebrew bin directory is {:?}", brew_bin.to_string_lossy());

    Ok(())
}
