use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("Quicksilver-Catalog.plist");
    let app_support_dir = standard_user.home_dir().join("Library/Application Support");
    let path = app_support_dir.join("Quicksilver-Catalog.plist");

    standard_user.as_user(|| {
        crate::fs::ensure_dir(&app_support_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        Ok(())
    })
}
