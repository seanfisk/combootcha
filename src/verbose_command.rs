use anyhow::{anyhow, Context, Result};
use log::info;

use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub(crate) fn run(command: &mut Command) -> Result<()> {
    run_base(command, None)
}

pub(crate) fn run_output(command: &mut Command) -> Result<Vec<u8>> {
    // TODO Switch to using subprocess library so that the subprocess can just inherit stderr. We don't want to capture it in 99% of the cases.
    log_command(&command, None);
    let output = command
        .output()
        .with_context(|| format!("Could not launch process {:?}", command))?;
    std::io::stderr().write_all(&output.stderr)?;
    check_status(command, &output.status)?;
    Ok(output.stdout)
}

// Accept the current_dir so that we can print it. If provided, this will override the Command's current_dir property.
// pub(crate) fn run_in_dir<P: AsRef<Path>>(command: &mut Command, current_dir: P) -> Result<()> {
//     run_base(command, Some(current_dir.as_ref()))
// }

fn run_base(command: &mut Command, current_dir: Option<&Path>) -> Result<()> {
    log_command(&command, current_dir);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    let status = command
        .status()
        .with_context(|| format!("Could not launch process {:?}", command))?;
    check_status(command, &status)
}

fn log_command(command: &Command, current_dir: Option<&Path>) {
    info!(
        "=> {:?}{}",
        command,
        current_dir.map_or("".to_owned(), |d| format!(" (cwd: {:?})", d))
    );
}

fn check_status(command: &Command, status: &ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Process {:?} failed with {}", command, status))
    }
}
