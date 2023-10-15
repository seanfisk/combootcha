use anyhow::Result;
use log::info;

use std::path::Path;

use crate::verbose_command::Command;

// Although it's possible to install to ~/Library/PreferencePanes and Network Link Conditioner *seems* to work if we do, it doesn't show up in System Preferences and must be launched via the "open" utility. So, install to the global location.

// TODO Not working under macOS Ventura
#[allow(dead_code)]
pub(crate) fn install() -> Result<()> {
    info!("Considering installing Network Link Conditioner");
    let prefix = Path::new("/Library/PreferencePanes");
    if prefix.join("Network Link Conditioner.prefPane").exists() {
        info!("Network Link Conditioner is already installed");
        return Ok(());
    }
    // See https://stackoverflow.com/a/9659486 for the download link.
    //
    // Since this is an Apple Developer download, we've vendored it into our repository.
    //
    // After installing the pref pane, this file was created by running the following command in the repo directory:
    //
    //     /usr/bin/tar -C /Library/PreferencePanes -cJvf 'src/network-link-conditioner.tar.xz' 'Network Link Conditioner.prefPane'
    //
    let bytes = include_bytes!("network-link-conditioner.tar.xz");
    Command::new("/usr/bin/tar")
        .arg("-x") // extract
        .arg("-J") // xz
        .arg("-v") // verbose
        .current_dir(prefix)
        .run_with_input(bytes)?;
    info!("Network Link Conditioner installed successfully");
    Ok(())
}
