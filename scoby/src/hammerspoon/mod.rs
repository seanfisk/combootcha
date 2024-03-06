use crate::UserExt as OtherUserExt;
use anyhow::Result;
use std::io::Write;
use users::{os::unix::UserExt, User};

pub(crate) fn configure(standard_user: &User, init_lua_extra_bytes: Option<&[u8]>) -> Result<()> {
    let bytes = include_bytes!("init.lua");
    let hammerspoon_dir = standard_user.home_dir().join(".hammerspoon");
    let path = hammerspoon_dir.join("init.lua");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&hammerspoon_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        if let Some(bytes) = init_lua_extra_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
        file.sync_all()?;
        Ok(())
    })
}
