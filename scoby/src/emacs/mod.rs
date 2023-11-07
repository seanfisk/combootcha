use crate::UserExt as OtherUserExt;
use anyhow::Result;
use std::io::Write;
use users::{os::unix::UserExt, User};

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let bytes = include_bytes!("spacemacs/init.el");
    // See https://develop.spacemacs.org/doc/DOCUMENTATION.html#alternative-dotdirectory
    let spacemacs_dir = standard_user.home_dir().join(".spacemacs.d");
    let path = spacemacs_dir.join("init.el");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&spacemacs_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    })
}
