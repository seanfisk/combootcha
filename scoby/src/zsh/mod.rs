use anyhow::Result;
use log::info;
use std::borrow::Cow;
use users::{os::unix::UserExt, User};

use crate::{text_buffer::TextBuffer, UserExt as OtherUserExt};

pub struct Config {
    profile: TextBuffer,
    rc: TextBuffer,
}

impl Config {
    pub(crate) fn new() -> Config {
        let mut profile = TextBuffer::new();
        profile.add_content(include_str!("profile.zsh"));
        let mut rc = TextBuffer::new();
        rc.add_content(include_str!("rc.zsh"));
        Self { profile, rc }
    }

    pub fn add_profile_content<T: Into<Cow<'static, str>>>(&mut self, text: T) -> &mut Self {
        self.profile.add_section(text);
        self
    }

    pub fn add_rc_content<T: Into<Cow<'static, str>>>(&mut self, text: T) -> &mut Self {
        self.rc.add_section(text);
        self
    }

    pub(crate) fn converge(&self, standard_user: &User) -> Result<()> {
        info!("Installing Zsh configuration files");
        let home_dir = standard_user.home_dir();
        standard_user.as_effective_user(|| {
            // Note: Don't use .zshenv because /etc/zprofile will clobber it
            for (file_name, buffer) in [(".zprofile", &self.profile), (".zshrc", &self.rc)] {
                let mut file = crate::fs::create_file(home_dir.join(file_name))?;
                buffer.to_writer(&mut file)?;
                file.sync_all()?;
            }

            Ok(())
        })
    }
}
