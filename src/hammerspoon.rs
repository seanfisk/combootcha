use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("hammerspoon-init.lua");
    let hammerspoon_dir = standard_user.home_dir().join(".hammerspoon");
    crate::fs::ensure_dir_with_owner(&hammerspoon_dir, &standard_user)?;
    let path = hammerspoon_dir.join("init.lua");

    {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
    }

    crate::fs::chown(path, &standard_user)
}
