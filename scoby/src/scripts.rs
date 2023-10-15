use anyhow::Result;
use users::{os::unix::UserExt, User};

use crate::user::UserExt as OtherUserExt;

pub(crate) fn install(standard_user: &User) -> Result<()> {
    let bin_dir = standard_user.home_dir().join("bin");
    standard_user.as_effective_user(|| crate::fs::ensure_dir(&bin_dir))
}
