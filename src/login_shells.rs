use anyhow::Result;
use log::{debug, info};
use users::{os::unix::UserExt, User};

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::path::PathExt;
use crate::verbose_command::Command;

pub(crate) fn set(standard_user: &User) -> Result<()> {
    info!("Querying Homebrew bin directory");
    let brew_prefix_output = Command::new("brew")
        .arg("--prefix")
        .user(standard_user)
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
    let username = standard_user.name().to_owned();
    let zsh_path_str = zsh_path.to_string_lossy();
    info!(
        "Considering setting login shell for user {:?} to {:?}",
        username, zsh_path_str
    );
    if standard_user.shell() == zsh_path {
        info!(
            "Login shell for user {:?} was already set to {:?}",
            username, zsh_path_str
        );
    } else {
        // Annoying that we have to clone this
        standard_user.clone().with_shell(&zsh_path);
        info!(
            "Set login shell for user {:?} to {:?}",
            username, zsh_path_str
        );
    }

    Ok(())
}
