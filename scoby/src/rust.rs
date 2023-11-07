use crate::verbose_command::Command;
use anyhow::Result;
use log::info;
use users::User;

pub(crate) fn configure(standard_user: User) -> Result<()> {
    Command::new("rustup-init")
        .arg("--no-modify-path") // I'll add ~/.cargo/bin to my shell profiles myself
        .arg("-y") // No prompts
        .args([
            "--components",
            "clippy", // Linter
            // Language Server Protocol implementation; used for Spacemacs
            // rls is another component that serves the same purpose but is deprecated in favor of rust-analyzer: https://blog.rust-lang.org/2022/07/01/RLS-deprecation.html
            "rust-analyzer",
        ])
        .user(standard_user)
        .run()?;

    info!("Rust installed with components successfully!");
    Ok(())
}
