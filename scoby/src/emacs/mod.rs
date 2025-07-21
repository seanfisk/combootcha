use crate::verbose_command::Command;
use crate::UserExt as OtherUserExt;
use anyhow::{Context, Result};
use log::info;
use std::io::Write;
use users::{os::unix::UserExt, User};

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let home_dir = standard_user.home_dir();

    {
        let install_dir = home_dir.join(".emacs.d");
        info!("Cloning Spacemacs into {install_dir:?}");
        // Couldn't find a solid way to clone idempotently. So technically this is a race condition, but in practice… come on. Not gonna happen.
        if install_dir
            .try_exists()
            .context("Checking whether Spacemacs is already cloned")?
        {
            info!("Spacemacs is already cloned");
        } else {
            Command::new(
                "/usr/local/bin/git", // Always use the Homebrew version
            )
            .args(["clone", "https://github.com/syl20bnr/spacemacs"])
            .arg(install_dir)
            .current_dir(home_dir) // Probably best to avoid running this from any old directory
            .user(standard_user.clone())
            .run()?;
        }
    }

    {
        let spacemacs_dir = home_dir.join(".spacemacs.d");
        info!("Installing Spacemacs customizations to {spacemacs_dir:?}");

        let bytes = include_bytes!("spacemacs/init.el");
        // See https://develop.spacemacs.org/doc/DOCUMENTATION.html#alternative-dotdirectory
        let install_path = spacemacs_dir.join("init.el");

        standard_user.as_effective_user(|| {
            crate::fs::ensure_dir(&spacemacs_dir)?;
            let mut file = crate::fs::create_file(&install_path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            Ok(())
        })
    }
}
