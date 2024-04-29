use anyhow::Result;
use log::info;
use std::os::unix::fs::OpenOptionsExt;
use users::{os::unix::UserExt, User};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::verbose_command::Command;
use crate::UserExt as OtherUserExt;

pub(crate) fn configure() -> Result<()> {
    // Disable auto-update. I have a Microsoft To Do task to periodically update Homebrew and installed formulae and casks.
    // Can be configured at the system, prefix, user, or shell level. Arbitrary for me since all are identical. Chose system as the most general.
    // https://docs.brew.sh/Manpage#environment
    let dir = Path::new("/etc/homebrew");
    crate::fs::ensure_dir(dir)?;
    let path = dir.join("brew.env");
    info!("Writing Homebrew configuration to {:?}", path);
    let mut file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o644)
        .open(path)?;
    writeln!(&mut file, "export HOMEBREW_NO_AUTO_UPDATE=1")?;
    file.sync_all()?;
    Ok(())
}

pub(crate) fn install_deps(standard_user: User, brewfile_extra_bytes: Option<&[u8]>) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    // This is the global Brewfile path (activated by --global)
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        file.write_all(include_bytes!("Brewfile"))?;
        if let Some(bytes) = brewfile_extra_bytes {
            file.write_all(b"\n")?;
            file.write_all(bytes)?;
        }
        file.sync_all()?;
        Ok(())
    })?;

    Command::new("brew")
        .arg("bundle")
        .arg("install")
        .arg("--verbose")
        .arg("--global")
        .user(standard_user)
        .run()
}
