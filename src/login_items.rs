use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

// Login items
//
// These are controlled by ~/Library/Preferences/com.apple.loginitems.plist,
// which is can be viewed in System Preferences > Users & Group > Current User >
// Login Items. However, this plist is difficult to edit manually because each
// item has an opaque key associated with it. Omitting the opaque key has
// yielded unpredicatable results, and the plist gets rewritten every time it is
// modified through the UI.
//
// Another solution is to create launch agents for each program. This is not as
// well-integrated with the macOS desktop experience, but seems to be the
// cleaner solution in the long run.
//
// See this StackOverflow question for more information:
// http://stackoverflow.com/q/12086638

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    let install_dir = standard_user.home_dir().join("Library/LaunchAgents");
    crate::fs::ensure_dir_with_owner(install_dir, &standard_user)?;
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
