use anyhow::{Context, Result};
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;
use std::path::Path;

use crate::user::UserExt as OtherUserExt;

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
    standard_user.as_effective_user(|| crate::fs::ensure_dir(&install_dir))?;
    for app in vec![
        "Flux",
        "Jettison",
        "Quicksilver",
        "gfxCardStatus",
        "iTerm",
        "Hammerspoon",
    ] {
        info!("Setting app {} to launch upon login", app);
        let label = format!("com.seanfisk.login.{}", app.to_lowercase());
        let agent_path = install_dir.join(format!("{}.plist", label));
        write_launch_agent(agent_path, &label, &standard_user, app)?;
    }
    Ok(())
}

fn write_launch_agent<P: AsRef<Path>>(
    path: P,
    label: &str,
    owner: &User,
    app_name: &str,
) -> Result<()> {
    use plist::Value;

    let mut dict = plist::dictionary::Dictionary::new();
    dict.insert("Key".to_owned(), Value::String(label.to_owned()));
    dict.insert(
        "ProgramArguments".to_owned(),
        Value::Array(vec![
            Value::String("/usr/bin/open".to_owned()),
            Value::String("-a".to_owned()),
            Value::String(app_name.to_owned()),
        ]),
    );
    dict.insert("RunAtLoad".to_owned(), Value::Boolean(true));
    owner.as_effective_user(|| {
        let mut file = crate::fs::create_file(path.as_ref())?;
        Value::Dictionary(dict).to_writer_xml(&mut file)?;
        // Add a trailing newline since the library doesn't do that
        writeln!(file).context("Writing trailing newline")
    })
}
