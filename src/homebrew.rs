use crate::verbose_command;
use anyhow::Result;
use std::os::unix::process::CommandExt;
use std::process::Command;
use users::User;

fn install_formulae(standard_user: User) -> Result<()> {
    for formula in vec!["git"] {
        verbose_command::run(
            Command::new("brew")
                .arg("install")
                .arg(formula)
                .uid(standard_user.uid()),
        )?;
    }

    Ok(())
}
