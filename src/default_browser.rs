use anyhow::Result;
use users::User;

use crate::verbose_command::Command;

pub(crate) fn set(user: &User) -> Result<()> {
    // What defaultbrowser does is pretty simple, but there really isn't a good reason to rewrite it into this program: https://github.com/kerma/defaultbrowser/blob/master/src/main.m
    Command::new("defaultbrowser")
        .arg("firefox")
        .user(user)
        .run()
}
