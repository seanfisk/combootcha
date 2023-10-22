use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;
use std::path::Path;

use crate::UserExt as OtherUserExt;

pub(crate) fn configure(
    standard_user: &User,
    profile_extra_bytes: Option<&[u8]>,
    rc_extra_bytes: Option<&[u8]>,
) -> Result<()> {
    info!("Installing Zsh configuration files");
    let home_dir = standard_user.home_dir();
    standard_user.as_effective_user(|| {
        install_dotfile(
            home_dir.join(".zprofile"),
            include_bytes!("profile.zsh"),
            profile_extra_bytes,
        )?;
        install_dotfile(
            home_dir.join(".zshrc"),
            include_bytes!("rc.zsh"),
            rc_extra_bytes,
        )
    })
}

fn install_dotfile<P: AsRef<Path>>(
    install_path: P,
    base_bytes: &[u8],
    extra_bytes: Option<&[u8]>,
) -> Result<()> {
    let install_path = install_path.as_ref();
    info!("Writing {install_path:?}");
    let mut file = crate::fs::create_file(install_path)?;
    file.write_all(base_bytes)?;
    if let Some(bytes) = extra_bytes {
        file.write_all(b"\n")?;
        file.write_all(bytes)?;
    }
    file.sync_all()?;
    Ok(())
}
