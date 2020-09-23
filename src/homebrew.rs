use crate::verbose_command;
use anyhow::Result;
use std::process::Command;

fn install_formulae(uid: u32) -> Result<()> {
    for formula in vec!["git"] {
        verbose_command::run(Command::new("brew").arg("install").arg(formula))?;
    }

    Ok(())
}
