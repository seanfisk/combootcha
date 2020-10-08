use anyhow::{Context, Result};
use log::info;
use nix::unistd::{seteuid, Uid};
use users::User;

pub(crate) trait UserExt {
    fn as_user<T, F: FnOnce() -> T>(&self, block: F) -> Result<T>;
}

impl UserExt for User {
    fn as_user<T, F: FnOnce() -> T>(&self, block: F) -> Result<T> {
        info!("Setting process effective user to {:?}", self.name());
        seteuid(Uid::from_raw(self.uid())).with_context(|| {
            format!("Could not set process effective user to {:?}", self.name())
        })?;
        let block_result = block();
        info!("Restoring process effective user to root");
        seteuid(Uid::from_raw(0)).context("Could not restore process effective user to root")?;
        Ok(block_result)
    }
}
