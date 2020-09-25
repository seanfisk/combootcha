use anyhow::{Context, Result};
use log::info;
use nix::unistd::Uid;
use users::User;

use std::path::Path;

pub(crate) fn chown<P: AsRef<Path>>(path: P, user: &User) -> Result<()> {
    let path_repr = path.as_ref().to_string_lossy();
    let user_repr = user.name();
    info!("Ensuring file {:?} is owned by {:?}", path_repr, user_repr);
    nix::unistd::chown(path.as_ref(), Some(Uid::from_raw(user.uid())), None).with_context(
        || {
            format!(
                "Could not change ownership of {:?} to user with name{:?}",
                path_repr, user_repr
            )
        },
    )?;
    Ok(())
}
