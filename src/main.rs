use anyhow::{anyhow, Error, Result};
use clap::{crate_authors, crate_description, crate_name, App, Arg};
use log::debug;
use users::get_user_by_name;

mod homebrew;
mod verbose_command;

fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

fn get_env(name: &str) -> Result<Option<String>> {
    use std::env::{var, VarError};
    match var(name) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            VarError::NotPresent => Ok(None),
            VarError::NotUnicode(_) => Err(Error::new(e)),
        },
    }
}

fn get_standard_username(cli_value: Option<&str>) -> Result<String> {
    debug!("Looking for standard user from CLI");
    match cli_value {
        Some(v) => {
            debug!("Standard user set to {:?} from command line", v);
            Ok(v.to_owned())
        }
        None => {
            debug!("Looking for standard user from SUDO_USER environment variable");
            match get_env("SUDO_USER")? {
                Some(v) => {
                    debug!("Standard user set to {:?} from SUDO_USER environment variable", v);
                    Ok(v)
                }
                None => Err(anyhow!("Standard user not given by --standard-user command-line option nor SUDO_USER environment variable")),
            }
        }
    }
}

fn main() -> Result<()> {
    if !is_root() {
        return Err(anyhow!("This program must be run as root!"));
    }

    let standard_user_arg = Arg::with_name("USERNAME")
        .short("u")
        .long("standard-user")
        .help("Standard user to run as; defaults to value of SUDO_USER environment variable")
        .takes_value(true);

    let app = App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(standard_user_arg);

    let matches = app.get_matches();

    let standard_username = get_standard_username(matches.value_of("USERNAME"))?;
    let standard_user = get_user_by_name(&standard_username).ok_or_else(|| {
        anyhow!(
            "User with name {:?} does not exist on this system!",
            standard_username
        )
    })?;

    homebrew::install_system(&standard_user)?;
    homebrew::install_deps(&standard_user)?;

    Ok(())
}
