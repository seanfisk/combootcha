use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;
use std::path::Path;

use crate::user::UserExt as OtherUserExt;
use crate::verbose_command::Command;

pub(crate) fn install_system(standard_user: &User) -> Result<()> {
    info!("Considering Homebrew installation");

    if Path::new("/usr/local/bin/brew").exists() {
        info!("Hombrew is already installed");
        Ok(())
    } else {
        info!("Installing Homebrew…");
        // Yeah, we could pull this down with reqwest, but it's a bit simpler to use the exact command that Hombrew provides
        Command::new("/bin/bash")
            .arg("-c")
            .arg("$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)")
            .user(&standard_user)
            .run()?;
        info!("Homebrew installed successfully");
        Ok(())
    }
}

pub(crate) fn install_deps(standard_user: &User) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    let bytes = include_bytes!("Brewfile");
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_user(|| {
        {
            let mut file = crate::fs::create_file(&path)?;
            file.write_all(bytes)?;
        }

        Command::new("brew")
            .arg("bundle")
            .arg("install")
            .arg("--verbose")
            .arg("--global")
            .run()
    })
}
