use anyhow::Result;
use std::borrow::Cow;
use users::{os::unix::UserExt, User};

use crate::{text_buffer::TextBuffer, UserExt as OtherUserExt};

pub struct Config {
    config: TextBuffer,
}

impl Config {
    pub(crate) fn new() -> Self {
        let mut config = TextBuffer::new();
        config.add_content(include_str!("config/pre"));
        Self { config }
    }

    pub fn add_config_content<T: Into<Cow<'static, str>>>(&mut self, text: T) -> &mut Self {
        self.config.add_section(text);
        self
    }

    pub(crate) fn converge(self, standard_user: &User) -> Result<()> {
        let mut config = self.config;
        config.add_content(include_str!("config/post"));

        let ssh_dir = standard_user.home_dir().join(".ssh");
        let path = ssh_dir.join("config");

        standard_user.as_effective_user(|| {
            crate::fs::ensure_dir(&ssh_dir)?;
            let mut file = crate::fs::create_file(&path)?;
            config.to_writer(&mut file)?;
            file.sync_all()?;
            Ok(())
        })
    }
}
