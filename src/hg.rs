use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;

// TODO Trying to go without this
#[allow(dead_code)]
pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("hgrc");
    let path = standard_user.home_dir().join(".hgrc");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        Ok(())
    })
}
