use anyhow::Result;
use clap::{Arg, ArgMatches};
use users::User;

use crate::verbose_command::Command;

const CLI_OPTION_NAME: &str = "set-default-browser";

pub(crate) fn cli_option<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name(CLI_OPTION_NAME)
        .short("-B")
        .long(CLI_OPTION_NAME)
        .help("Set the default browser (shows a prompt every time)")
}

pub(crate) fn configure(
    matches: &ArgMatches,
    user: &User, // We'll clone on use
) -> Result<()> {
    if matches.is_present(CLI_OPTION_NAME) {
        // What defaultbrowser does is pretty simple, but there really isn't a good reason to rewrite it into this program: https://github.com/kerma/defaultbrowser/blob/master/src/main.m
        Command::new("defaultbrowser")
            .arg("firefox")
            .user(user.clone())
            .run()?;
    }
    Ok(())
}
