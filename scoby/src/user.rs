use anyhow::{Context, Result};
use log::debug;
use nix::unistd::{seteuid, Uid};
use users::User;

pub trait UserExt {
    // Just force the block to return Result so that we don't have to deal with a nested Result
    fn as_effective_user<T, F: FnOnce() -> Result<T>>(&self, block: F) -> Result<T>;
}

impl UserExt for User {
    fn as_effective_user<T, F: FnOnce() -> Result<T>>(&self, block: F) -> Result<T> {
        debug!("Setting process effective user to {:?}", self.name());
        seteuid(Uid::from_raw(self.uid())).with_context(|| {
            format!("Could not set process effective user to {:?}", self.name())
        })?;
        let block_result = block();
        debug!("Restoring process effective user to root");
        seteuid(Uid::from_raw(0)).context("Could not restore process effective user to root")?;
        block_result
    }
}
