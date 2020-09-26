use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::path::Path;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bgs_dir = standard_user
        .home_dir()
        .join(Path::new("Library/Application Support/iTerm2/Backgrounds"));
    crate::fs::ensure_dir_with_owner(bgs_dir, &standard_user)?;
    Ok(())
}
