use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::process::{Command, Output};

// dscacheutil will print no output and exit with a 0 code if it doesn't find
// the specified name. This is really unclear in the context of using it for DNS
// lookups, so we are overriding it.

pub fn dscacheutil(
    category: &str,
    key: &str,
    value: &str,
    operation_description: &str,
) -> Result<()> {
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new("/usr/bin/dscacheutil")
        .arg("-q")
        .arg(category)
        .arg("-a")
        .arg(key)
        .arg(value)
        .output()?;

    if status.success() {
        if stdout.is_empty() {
            Err(anyhow!("Could not {}: {}", operation_description, value,))
        } else {
            std::io::stdout()
                .write_all(&stdout)
                .context("Writing dscacheutil output to stdout")?;
            Ok(())
        }
    } else {
        let message = std::str::from_utf8(&stderr)
            .context("Failure converting dscacheutil stderr to UTF-8")?;
        Err(anyhow!(
            "dscacheutil failed with message: {}",
            message.trim()
        ))
    }
}
