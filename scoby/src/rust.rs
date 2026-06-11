use crate::verbose_command::Command;
use anyhow::Result;
use log::info;
use std::path::Path;
use users::User;

pub(crate) fn configure(standard_user: User) -> Result<()> {
    // Rustup is keg-only, so use an absolute path to ensure we're using Rustup from Homebrew.
    let rustup_bin = Path::new("/usr/local/opt/rustup/bin");

    let components: [&str; 2] = [
        // Linter
        "clippy",
        // Language Server Protocol implementation; used for Spacemacs
        // rls is another component that serves the same purpose but is deprecated in favor of rust-analyzer: https://blog.rust-lang.org/2022/07/01/RLS-deprecation.html
        "rust-analyzer",
    ];

    info!("Installing the Rust toolchain using rustup with components {components:?}");

    Command::new(rustup_bin.join("rustup"))
        .args(["toolchain", "install"])
        .arg("--component")
        .arg(components.join(","))
        .arg("stable")
        .user(standard_user.clone())
        .run()?;

    info!("Rust installed or updated successfully!");

    info!("Registering Homebrew's Rust with Rustup");

    // https://rust-lang.github.io/rustup/installation/already-installed-rust.html
    // This command is idempotent
    Command::new(rustup_bin.join("rustup"))
        .args(["toolchain", "link", "system", "/usr/local"])
        .user(standard_user)
        .run()?;

    info!("Successfully registered Homebrew's Rust with Rustup!");

    Ok(())
}
