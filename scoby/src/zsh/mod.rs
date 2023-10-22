use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::UserExt as OtherUserExt;

pub(crate) fn configure(standard_user: User, rc_extra_bytes: Option<&[u8]>) -> Result<()> {
    info!("Installing Zsh configuration files");
    let rc_path = standard_user.home_dir().join(".zshrc");
    
    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        {
            let bytes = include_bytes!("rc.zsh");
            file.write_all(bytes)?;
        }
        if let Some(bytes) = rc_extra_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
        Ok(())
    })?;
}
