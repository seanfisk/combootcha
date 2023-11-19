use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: &User, config_extra_bytes: Option<&[u8]>) -> Result<()> {
    let ssh_dir = standard_user.home_dir().join(".ssh");
    let path = ssh_dir.join("config");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&ssh_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(include_bytes!("config/pre"))?;
        if let Some(bytes) = config_extra_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
        file.write_all(b"\n")?;
        file.write_all(include_bytes!("config/post"))?;
        file.sync_all()?;
        Ok(())
    })
}
