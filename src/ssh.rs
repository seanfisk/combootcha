use anyhow::Result;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;
use crate::Config;

pub(crate) fn configure(config: Config, standard_user: &User) -> Result<()> {
    let ssh_dir = standard_user.home_dir().join(".ssh");
    let path = ssh_dir.join("config");

    standard_user.as_effective_user(|| {
        crate::fs::ensure_dir(&ssh_dir)?;
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(include_bytes!("ssh-config/shared"))?;
        if config == Config::work {
            file.write(b"\n")?;
            file.write_all(include_bytes!("ssh-config/work"))?;
        }
        Ok(())
    })
}
