use anyhow::{anyhow, Result};
use log::{debug, info};
use users::User;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    for shell in vec!["bash", "zsh"] {
        let shell_path = brew_bin.join(shell);
        info!(
            "Considering adding {:?} to {:?}",
            shell_path.to_string_lossy(),
            shells_config_path.to_string_lossy()
        );
        let shell_path_str = shell_path
            .to_str()
            .ok_or_else(|| anyhow!("Error converting shell path {:?} to a string", shell_path))?;
        if shells.contains(shell_path_str) {
            info!(
                "Shell {:?} is already listed in {:?}",
                shell_path.to_string_lossy(),
                shells_config_path.to_string_lossy()
            );
        }
    }

    Ok(())
}
