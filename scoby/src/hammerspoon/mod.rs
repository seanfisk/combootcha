use crate::{text_buffer::TextBuffer, UserExt as OtherUserExt};
use anyhow::Result;
use std::borrow::Cow;
use users::{os::unix::UserExt, User};

pub struct Config {
    init_lua: TextBuffer,
}

impl Config {
    pub(crate) fn new() -> Self {
        let mut init_lua = TextBuffer::new();
        init_lua.add_content(include_str!("init.lua"));
        Self { init_lua }
    }

    pub fn add_init_lua_content<T: Into<Cow<'static, str>>>(&mut self, text: T) -> &mut Self {
        self.init_lua.add_section(text);
        self
    }

    pub(crate) fn configure(&self, standard_user: &User) -> Result<()> {
        let hammerspoon_dir = standard_user.home_dir().join(".hammerspoon");
        let path = hammerspoon_dir.join("init.lua");

        standard_user.as_effective_user(|| {
            crate::fs::ensure_dir(&hammerspoon_dir)?;
            let mut file = crate::fs::create_file(&path)?;
            self.init_lua.to_writer(&mut file)?;
            file.sync_all()?;
            Ok(())
        })
    }
}
