use crate::verbose_command::Command;
use anyhow::Result;
use log::info;

pub(crate) fn configure() -> Result<()> {
    info!("Configuring power management preferences");
    let settings: [(&str, &str); 1] = [
        ("womp", "0"),
    ];
    for (name, value) in settings {
        info!("pmset {:?} to {:?}", name, value);
    }

    Ok(())

    // pmset(settings)
}

// struct Setting<'a> {
//     name: &'a str,
//     value: &'a str
// }

// fn pmset(settings: I) -> Result
// where
//     I: IntoIterator,
//     I::Item: (str, str),
// {
//     for setting in settings {
//         info!("pmset {:?} to {:?}", setting.name, setting.value);
//     }
//     Ok(())
//     // let mut command = Command::new("git");
//     // command.current_dir(user.home_dir()); // Running in a repo shouldn't be a problem, but let's not do it anyway
//     // command.user(user);
//     // command
// }
