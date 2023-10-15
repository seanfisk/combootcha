use anyhow::{Context, Result};
use log::info;

use std::path::Path;

pub(crate) fn create_file<P: AsRef<Path>>(path: P) -> Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    info!("Creating file {:?}", path.as_ref().to_string_lossy());
    Ok(std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(path.as_ref())?)
}

pub(crate) fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    info!("Ensuring directory {:?} exists", path.to_string_lossy());
    std::fs::create_dir_all(path)
        .with_context(|| format!("Could not create directory {:?}", path.to_string_lossy()))
}
