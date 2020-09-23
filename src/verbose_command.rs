use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::process::Command;

pub fn run(command: &mut Command) -> Result<()> {
    run_base(command, None)
}

// Accept the current_dir so that we can print it. If provided, this will override the Command's current_dir property.
pub fn run_in_dir<P: AsRef<Path>>(command: &mut Command, current_dir: P) -> Result<()> {
    run_base(command, Some(current_dir.as_ref()))
}

fn run_base(command: &mut Command, current_dir: Option<&Path>) -> Result<()> {
    println!(
        "=> {:?}{}",
        command,
        current_dir.map_or("".to_owned(), |d| format!(" (cwd: {:?})", d))
    );
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    let status = command.status().context("Could not launch process")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Process failed with status {}", status))
    }
}