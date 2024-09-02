use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::UserExt as OtherUserExt;

pub struct Config {
    profile_content: String,
    rc_content: String,
}

fn add_content<C: AsRef<str>>(buffer: &mut String, new_content: C) {
    buffer.push('\n'); // Add a blank line between existing and new content
    buffer.push_str(new_content.as_ref());
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            profile_content: include_str!("profile.zsh").to_owned(),
            rc_content: include_str!("rc.zsh").to_owned(),
        }
    }

    pub fn add_profile_content<C: AsRef<str>>(&mut self, content: C) {
        add_content(&mut self.profile_content, content);
    }

    pub fn add_rc_content<C: AsRef<str>>(&mut self, content: C) {
        add_content(&mut self.rc_content, content);
    }

    pub(crate) fn converge(self, standard_user: &User) -> Result<()> {
        info!("Installing Zsh configuration files");
        let home_dir = standard_user.home_dir();
        standard_user.as_effective_user(|| {
            // Note: Don't use .zshenv because /etc/zprofile will clobber it
            for (file_name, content) in [
                (".zprofile", self.profile_content),
                (".zshrc", self.rc_content),
            ] {
                let mut file = crate::fs::create_file(home_dir.join(file_name))?;
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
            }

            Ok(())
        })
    }
}
