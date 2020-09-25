use anyhow::Result;
use log::{debug, info};
use users::User;

// use std::ffi::OsStr;
// use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::fs::read_to_string;

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

    let shells_config_path = Path::new("/etc/shells");
    debug!("Reading {:?}", shells_config_path.to_string_lossy());
    let shells_config_lines = read_to_string(shells_config_path)?.lines();

    for shell in vec!["bash", "zsh"] {
        let shell_path = brew_bin.join(shell);
        info!("Considering adding {:?} to the shells config file", shell_path.to_string_lossy());
        
    }

    Ok(())
}
