use anyhow::Result;
use log::info;
use users::User;

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    for app in vec![
        "Flux",
        "Jettison",
        "Quicksilver",
        "gfxCardStatus",
        "iTerm",
        "Hammerspoon",
    ] {
        info!("Setting app {} to launch upon login", app);
    }
    Ok(())
}
