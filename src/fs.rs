use anyhow::{Context, Result};
use log::info;
use nix::unistd::Uid;
use users::User;

use std::path::Path;

use crate::verbose_command::Command;

pub(crate) fn create_file<P: AsRef<Path>>(path: P) -> Result<std::fs::File> {
    info!("Creating file {:?}", path.as_ref().to_string_lossy());
    use std::os::unix::fs::OpenOptionsExt;
    Ok(std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o400)
        .open(path.as_ref())?)
}

// pub(crate) fn ensure_containing_dir<P: AsRef<Path>>(path: P) -> Result<()> {
//     std::fs::create_dir_all(safe_parent(path))?;
//     Ok(())
// }

pub(crate) fn ensure_dir_with_owner<P: AsRef<Path>>(path: P, owner: &User) -> Result<()> {
    info!(
        "Ensuring directory {:?} exists",
        path.as_ref().to_string_lossy()
    );
    // Rather than ensuring all ancestors exist and carefully chowning them only when they are created, it is much easier simply to create the directory as the user themselves.
    Command::new("mkdir")
        .args(vec!["-m", "u=rwx"])
        .arg("-p")
        .arg("-v")
        .arg("--")
        .arg(path.as_ref().as_os_str())
        .user(&owner)
        .run()
}

// fn safe_parent<P: AsRef<Path>>(path: P) -> PathBuf {
//     path.as_ref().parent().map_or_else(|| PathBuf::from("/"), |p| p.to_owned())
// }

pub(crate) fn chown<P: AsRef<Path>>(path: P, user: &User) -> Result<()> {
    let path_repr = path.as_ref().to_string_lossy();
    let user_repr = user.name();
    info!("Ensuring file {:?} is owned by {:?}", path_repr, user_repr);
    nix::unistd::chown(path.as_ref(), Some(Uid::from_raw(user.uid())), None).with_context(
        || {
            format!(
                "Could not change ownership of {:?} to user with name {:?}",
                path_repr, user_repr
            )
        },
    )?;
    Ok(())
}
