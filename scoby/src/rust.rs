use crate::verbose_command::Command;
use anyhow::Result;
use log::info;
use users::User;

pub(crate) fn configure(standard_user: User) -> Result<()> {
    Command::new("rustup")
        .args(["component", "add"])
        .args([
            "clippy", // Linter
            "rls",    // Language Server Protocol implementation; used for Spacemacs
        ])
        .user(standard_user)
        .run()?;

    info!("Rust components added successfully");
    Ok(())
}
