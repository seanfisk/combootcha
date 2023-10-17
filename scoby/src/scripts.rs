use anyhow::Result;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use users::{os::unix::UserExt, User};

use crate::UserExt as OtherUserExt;

macro_rules! install_script {
    ($install_dir : expr, $name : literal) => {
        install_script(
            $install_dir,
            $name,
            include_bytes!(concat!("../scripts/target/release/", $name)),
        )
    };
}

pub(crate) fn install(standard_user: &User) -> Result<()> {
    standard_user.as_effective_user(|| {
        let bin_dir = standard_user.home_dir().join("bin");
        crate::fs::ensure_dir(&bin_dir)?;

        install_script!(&bin_dir, "dns")?;
        install_script!(&bin_dir, "rdns")?;

        Ok(())
    })
}

fn install_script(install_dir: &Path, name: &str, contents: &[u8]) -> Result<()> {
    let path = install_dir.join(name);
    info!("Writing script to {path:?}");

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o755)
        .open(path)?
        .write_all(contents)?;

    Ok(())
}
