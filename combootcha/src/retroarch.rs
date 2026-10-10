use anyhow::Result;
use users::User;

use scoby::UserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    standard_user.as_effective_user(|| Ok(()))
}
