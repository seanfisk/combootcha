use crate::verbose_command::Command;
use anyhow::Result;
use log::info;

pub(crate) fn configure() -> Result<()> {
    info!("Configuring power management preferences");
    Command::new("pmset")
        .arg("-a") // Apply to all states (battery, charger, & UPS)
        .args(["womp", "0"]) // Disable 'Wake for network access'
        .args(["powernap", "0"]) // Disable Power Nap, which awakens the computer to check email, etc. Annoying.
        .args(["acwake", "0"]) // Don't wake up when charger is plugged in
        .args(["lidwake", "1"]) // Wake up when lid is opened
        .run()
}
