use anyhow::Result;
use log::{debug, info};
use users::User;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::verbose_command::Command;
use crate::PathExt;

pub(crate) fn set(standard_user: User) -> Result<()> {
    info!("Querying Homebrew bin directory");
    let brew_prefix_output = Command::new("brew")
        .arg("--prefix")
        .user(standard_user.clone())
        .output()?;
    let brew_prefix = Path::new(std::str::from_utf8(&brew_prefix_output)?.trim_end_matches('\n'));
    let brew_bin = brew_prefix.join("bin");

    info!("Homebrew bin directory is {:?}", brew_bin.to_string_lossy());

    let shells_config_path = Path::new("/etc/shells");
    debug!("Reading {:?}", shells_config_path.to_string_lossy());
    let shells = {
        let mut set = HashSet::new();
        for line in BufReader::new(File::open(shells_config_path)?).lines() {
            let line = line?;
            if !line.is_empty() && !line.trim_start().starts_with('#') {
                set.insert(line);
            }
        }
        set
    };

    for shell in ["bash", "zsh"] {
        let shell_path = brew_bin.join(shell);
        info!(
            "Considering adding {:?} to {:?}",
            shell_path.to_string_lossy(),
            shells_config_path.to_string_lossy()
        );
        let shell_path_str = shell_path.to_str_safe()?;
        if shells.contains(shell_path_str) {
            info!(
                "Shell {:?} is already listed in {:?}",
                shell_path.to_string_lossy(),
                shells_config_path.to_string_lossy()
            );
        }
    }

    let zsh_path = brew_bin.join("zsh");
    let username = standard_user.name();
    let zsh_path_str = zsh_path.to_string_lossy();
    info!(
        "Setting login shell for user {:?} to {:?}",
        username, zsh_path_str
    );
    // Note that User#with_shell only sets the shell for that struct within Rust. It does NOT update the backend user database. For that we have to use chsh(1).
    //
    // Runing chsh as the standard user will result in a password prompt. We don't want that so we will run as root.
    //
    // chsh is already idempotent so just run it every time. No need to look before we leap.
    Command::new("/usr/bin/chsh")
        .arg("-s")
        .arg(zsh_path)
        .arg(username) // Must be username, not uid
        .run()
}
