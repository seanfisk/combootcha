use anyhow::Result;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use users::{os::unix::UserExt, User};

use crate::UserExt as OtherUserExt;

pub(crate) fn install(standard_user: &User) -> Result<()> {
    standard_user.as_effective_user(|| {
        let bin_dir = standard_user.home_dir().join("bin");
        crate::fs::ensure_dir(&bin_dir)?;

        {
            let path = bin_dir.join("dns");
            info!("Writing script to {path:?}");
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o755)
                .open(path)?;
            file.write_all(include_bytes!("../scripts/target/release/dns"))?;
        }

        Ok(())
    })
}
