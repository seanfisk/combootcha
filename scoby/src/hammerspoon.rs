use crate::user::UserExt as OtherUserExt;
use anyhow::Result;
use std::io::Write;
use users::{os::unix::UserExt, User};

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("hammerspoon-init.lua");
    let hammerspoon_dir = standard_user.home_dir().join(".hammerspoon");
    let path = hammerspoon_dir.join("init.lua");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&hammerspoon_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        Ok(())
    })
}
