use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;
use crate::verbose_command::Command;
use crate::Config;

pub(crate) fn install_deps(config: Config, standard_user: &User) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    // This is the global Brewfile path (activated by --global)
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        {
            let bytes = include_bytes!("brewfiles/shared.rb");
            file.write_all(bytes)?;
        }
        match config {
            Config::personal => {
                let bytes = include_bytes!("brewfiles/personal.rb");
                file.write_all(bytes)?;
            }
            Config::work => {
                let bytes = include_bytes!("brewfiles/work.rb");
                file.write_all(bytes)?;
            }
        };

        Ok(())
    })?;

    Command::new("brew")
        .arg("bundle")
        .arg("install")
        .arg("--verbose")
        .arg("--global")
        .user(standard_user)
        .run()
}
