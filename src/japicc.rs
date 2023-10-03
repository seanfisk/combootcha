use anyhow::{Context, Result};
use log::info;

use std::path::Path;

use crate::path::PathExt;
use crate::verbose_command::Command;

pub(crate) fn install() -> Result<()> {
    let prefix = Path::new("/usr/local");

    info!("Considering installing japi-compliance-checker");
    if prefix.join("bin/japi-compliance-checker").exists() {
        info!("japi-compliance-checker is already installed");
        return Ok(());
    }

    let version = "2.4";
    let url = format!(
        "https://github.com/lvc/japi-compliance-checker/archive/{}.tar.gz",
        version
    );
    let bytes = reqwest::blocking::get(url)
        .context("Downloading japi-compliance-checker")?
        .error_for_status()
        .context("HTTP server reported failure")?
        .bytes()
        .context("Getting japi-compliance-checker archive bytes")?;
    let temp_dir = tempfile::tempdir()
        .context("Creating temporary directory for building japi-compliance-checker")?;

    Command::new("/usr/bin/tar")
        .arg("-x") // extract
        .arg("-z") // gz
        .arg("-v") // verbose
        .cwd(&temp_dir.path())
        .run_with_input(&bytes)?;

    // The Makefile is broken and should have the 'install' target marked as phony, but doesn't. This causes the target not to run. Just run what it would have executed anyway.
    Command::new("/usr/local/bin/perl")
        .arg("Makefile.pl")
        .arg("-install")
        .args(&["-prefix", prefix.to_str_safe()?])
        .cwd(
            temp_dir
                .path()
                .join(format!("japi-compliance-checker-{}", version)),
        )
        .run()?;

    temp_dir
        .close()
        .context("Deleting temporary directory for building japi-compliance-checker")?;

    // Ensure the modules are accesible by a standard user
    Command::new("chmod")
        .arg("-R")
        .arg("go+rX")
        .arg(prefix.join("share/japi-compliance-checker").to_str_safe()?)
        .run()?;

    info!("japi-compliance-checker installed successfully");

    Ok(())
}
