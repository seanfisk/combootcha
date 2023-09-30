use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

use std::io::Write;

use crate::user::UserExt as OtherUserExt;
use crate::verbose_command::Command;
use crate::Config;

// What I am going to do for now is just instruct the user (me) to install Homebrew manually. It should be a one-and-done thing and then I don't have to worry about use of sudo or non-interactive stuff.

// pub(crate) fn install_system(standard_user: &User) -> Result<()> {
//     info!("Considering Homebrew installation");

//     if Path::new("/usr/local/bin/brew").exists() {
//         info!("Hombrew is already installed");
//         Ok(())
//     } else {
//         info!("Installing Homebrew…");
//         // Yeah, we could pull this down with reqwest, but it's a bit simpler to use the exact command that Hombrew provides
//         // TODO I think we should consider making this simpler (and possibly using reqwest); the nested shell execution is a bit ugly
//         Command::new("/bin/bash")
//             .arg("-c")
//             .arg("NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"")
//             .user(&standard_user)
//             .run()?;
//         info!("Homebrew installed successfully");
//         Ok(())
//     }
// }

pub(crate) fn install_deps(config: Config, standard_user: &User) -> Result<()> {
    info!("Installing Homebrew dependencies via Brewfile");
    let path = standard_user.home_dir().join(".Brewfile");

    standard_user.as_effective_user(|| {
        let mut file = crate::fs::create_file(&path)?;
        {
            let bytes = include_bytes!("brewfiles/shared.rb");
            file.write_all(bytes)?;
        }
        match config {
            Config::personal => {
                let bytes = include_bytes!("brewfiles/personal.rb");
                file.write_all(bytes)?;
            }
            Config::work => {
                let bytes = include_bytes!("brewfiles/work.rb");
                file.write_all(bytes)?;
            }
        };

        Ok(())
    })?;

    Command::new("brew")
        .arg("bundle")
        .arg("install")
        .arg("--verbose")
        .arg("--global")
        .user(&standard_user)
        .run()
}
