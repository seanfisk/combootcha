use crate::verbose_command::Command;
use anyhow::Result;
use log::info;

pub(crate) fn configure() -> Result<()> {
    info!("Configuring the Application Layer Firewall");
    // Since we're using the firewall's CLI (instead of, e.g. modifying a plist directly), the daemon will reload automatically
    Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .args(["--setglobalstate", "on"]) // Enable the firewall
        .args(["--setblockall", "on"]) // Block all non-essential incoming connections
        .args(["--setstealthmode", "off"]) // Acknowledge existence via ICMP
        .args(["--setallowsigned", "on"]) // Allow built-in signed software to accept incoming connections
        .args(["--setallowsignedapp", "on"]) // Allow downloaded signed software to accept incoming connections
        .run()
}
