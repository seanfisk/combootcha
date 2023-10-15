use crate::verbose_command::Command;
use anyhow::Result;
use log::info;

pub(crate) fn configure() -> Result<()> {
    info!("Configuring power management preferences");
    let mut command = Command::new("pmset");
    command.arg("-a"); // Apply to all states (battery, charger, & UPS)
    for (name, value) in [
        ("womp", "0"),     // Disable 'Wake for network access'
        ("powernap", "0"), // Disable Power Nap, which awakens the computer to check email, etc. Annoying.
        ("acwake", "0"),   // Don't wake up when charger is plugged in
        ("lidwake", "1"),  // Wake up when lid is opened
    ] {
        command.arg(name);
        command.arg(value);
    }
    command.run()?;
    Ok(())
}
