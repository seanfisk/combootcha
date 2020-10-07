use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

pub(crate) fn install_preferences(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("Quicksilver-Catalog.plist");
    let path = standard_user
        .home_dir()
        .join("Library/Application Support/Quicksilver-Catalog.plist");

    {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
    }

    crate::fs::chown(path, &standard_user)
}
