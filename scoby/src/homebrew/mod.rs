use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::verbose_command::Command;
use crate::UserExt as OtherUserExt;

pub(crate) fn install_deps(standard_user: User, brewfile_extra_bytes: Option<&[u8]>) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    // This is the global Brewfile path (activated by --global)
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(include_bytes!("Brewfile"))?;
        if let Some(bytes) = brewfile_extra_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
        file.sync_all()?;
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
