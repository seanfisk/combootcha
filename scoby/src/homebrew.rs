use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;
use crate::verbose_command::Command;
use crate::Config;

pub(crate) fn install_deps(standard_user: User, brewfile_extra_bytes: Option<&[u8]>) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    // This is the global Brewfile path (activated by --global)
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        {
            let bytes = include_bytes!("Brewfile");
            file.write_all(bytes)?;
        }
        if let Some(bytes) = extra_brewfile_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
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
