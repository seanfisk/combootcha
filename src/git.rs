use anyhow::Result;
use users::User;

use crate::verbose_command::Command;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let c = Gitconfig::new(&standard_user);
    c.section(&["user"])
        .set("name", "Sean Fisk")?
        .set("email", "sean@seanfisk.com")?;
    c.section(&["core"])
        .set("excludesfile", "~/.gitignore-global")?;
    Ok(())
}

struct Gitconfig<'a> {
    user: &'a User,
}

impl<'a> Gitconfig<'a> {
    fn new(user: &'a User) -> Gitconfig {
        Gitconfig { user: user }
    }

    fn section(&self, path: &'a [&'a str]) -> Section<'a> {
        Section::new(&self.user, &path)
    }
}

struct Section<'a> {
    path: &'a [&'a str],
    user: &'a User,
}

impl<'a> Section<'a> {
    fn new(user: &'a User, path: &'a [&'a str]) -> Section<'a> {
        Section {
            path: path,
            user: user,
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<&Section> {
        let dotted_path = self
            .path
            .iter()
            .chain(std::iter::once(&key))
            .cloned()
            .collect::<Vec<_>>()
            .join(".");
        Command::new("git")
            .arg("config")
            .arg("--global")
            .arg("--")
            .arg(dotted_path)
            .arg(value)
            .user(self.user)
            .run()?;
        Ok(self)
    }
}
