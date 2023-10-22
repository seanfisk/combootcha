use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::UserExt as OtherUserExt;

// TODO Trying to go without this app
#[allow(dead_code)]
pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("catalog.plist");
    let app_support_dir = standard_user.home_dir().join("Library/Application Support");
    let path = app_support_dir.join("Quicksilver-Catalog.plist");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&app_support_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        Ok(())
    })
}
