use anyhow::Result;
use std::process::Command;
use crate::verbose_command;

fn install_formulae(uid: u32) -> Result<()> {
    for formula in vec![
        "git",
    ] {
        verbose_command::run(
            Command::new("brew")
                .arg("install")
                .arg(formula)
                )?;
    }

    Ok(())
}
