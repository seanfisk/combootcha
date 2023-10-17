// dscacheutil will print no output and exit with a 0 code if it doesn't find
// the specified name. This is really unclear in the context of using it for DNS
// lookups, so we are overriding it.

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::io::Write;
use std::process::{Command, Output};

#[derive(Parser, Debug)]
#[command(
    author,
    about = "Resolve a domain name on macOS using the native DNS facilities."
)]
struct Args {
    #[arg(help = "Domain name to resolve")]
    domain_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let domain_name = args.domain_name;

    let mut command = Command::new("/usr/bin/dscacheutil");
    command
        .args(&["-q", "host", "-a", "name"])
        .arg(&domain_name);
    let Output {
        status,
        stdout,
        stderr,
    } = command.output()?;

    if status.success() {
        if stdout.is_empty() {
            Err(anyhow!("Could not resolve domain name: {}", domain_name))
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
