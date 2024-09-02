use anyhow::Result;
use log::info;
use std::borrow::Cow;
use std::os::unix::fs::OpenOptionsExt;
use users::{os::unix::UserExt, User};

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::text_buffer::TextBuffer;
use crate::verbose_command::Command;
use crate::UserExt as OtherUserExt;

pub struct Config {
    global_brewfile: TextBuffer,
}

impl Config {
    pub(crate) fn new() -> Self {
        let mut global_brewfile = TextBuffer::new();
        global_brewfile.add_content(include_str!("Brewfile"));
        Self { global_brewfile }
    }

    pub fn add_global_brewfile_content<T: Into<Cow<'static, str>>>(&mut self, text: T) {
        self.global_brewfile.add_section(text)
    }

    pub(crate) fn converge(&self, standard_user: User, install_deps: bool) -> Result<()> {
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

        // This is the global Brewfile path (activated by --global)
        let path = standard_user.home_dir().join(".Brewfile");

        standard_user.as_effective_user(|| {
            let mut file = crate::fs::create_file(&path)?;
            self.global_brewfile.to_writer(&mut file)?;
            file.sync_all()?;
            Ok(())
        })?;

        if install_deps {
            info!("Installing Homebrew dependencies via Brewfile");

            Command::new("brew")
                .arg("bundle")
                .arg("install")
                .arg("--verbose")
                .arg("--global")
                .user(standard_user)
                .run()?;
        }

        Ok(())
    }
}
