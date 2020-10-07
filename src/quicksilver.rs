use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("Quicksilver-Catalog.plist");
    let app_support_dir = standard_user.home_dir().join("Library/Application Support");
    crate::fs::ensure_dir_with_owner(&app_support_dir, &standard_user)?;
    let path = app_support_dir.join("Quicksilver-Catalog.plist");

    {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
    }

    crate::fs::chown(path, &standard_user)
}
