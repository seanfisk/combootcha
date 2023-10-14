use crate::verbose_command::Command;
use anyhow::{Context, Result};
use log::info;
use std::path::Path;
use users::User;

pub(crate) fn install(standard_user: User) -> Result<()> {
    info!("Considering installing Cathode terminal emulator");

    let install_dir = Path::new("/Applications");
    let install_path = install_dir.join("Cathode.app");
    if install_path
        .try_exists()
        .context("Checking whether Cathode is installed")?
    {
        info!("Cathode is already installed");
        return Ok(());
    }

    let bytes = include_bytes!("Cathode-2.4.1.tar.xz");
    Command::new("/usr/bin/tar")
        .arg("-x") // extract
        .arg("-J") // xz
        .arg("-v") // verbose
        .current_dir(install_dir)
        .user(standard_user)
        .run_with_input(bytes)?;

    info!("Cathode installed successfully");
    Ok(())
}
