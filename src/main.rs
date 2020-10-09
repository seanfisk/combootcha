mod env;
mod fs;
mod hammerspoon;
mod homebrew;
mod iterm2;
mod japicc;
mod karabiner;
mod logging;
mod login_items;
mod login_shells;
mod network_link_conditioner;
mod path;
mod preferences;
mod quicksilver;
mod scripts;
mod user;
mod user_defaults;
mod verbose_command;

use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_description, crate_name, App, AppSettings, Arg};
use log::{debug, info};
use logging::ColorMode;
use users::get_user_by_name;

fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
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
            match env::get("SUDO_USER")? {
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

    let log_level_arg = Arg::with_name("log-level")
        .short("l")
        .possible_values(&logging::LogLevel::variants())
        .help("Set the minimum log level")
        .long("log-level")
        .takes_value(true)
        .value_name("LEVEL");

    let standard_user_arg = Arg::with_name("USERNAME")
        .short("u")
        .long("standard-user")
        .help("Standard user to run as; defaults to value of SUDO_USER environment variable")
        .takes_value(true);

    let color_mode = logging::read_color_mode_from_env()?;

    let app = App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .setting(AppSettings::ColoredHelp)
        .setting(match color_mode {
            ColorMode::Never => AppSettings::ColorNever,
            ColorMode::Always => AppSettings::ColorAlways,
            ColorMode::Auto => AppSettings::ColorAuto,
        })
        .arg(log_level_arg)
        .arg(standard_user_arg);

    let matches = app.get_matches();

    logging::init(color_mode, matches.value_of("log-level"))?;
    debug!("Logger was succesfully instantiated");

    let standard_username = get_standard_username(matches.value_of("USERNAME"))?;
    let standard_user = get_user_by_name(&standard_username).ok_or_else(|| {
        anyhow!(
            "User with name {:?} does not exist on this system!",
            standard_username
        )
    })?;

    // homebrew::install_system(&standard_user)?;
    // homebrew::install_deps(&standard_user)?;

    login_shells::set(&standard_user)?;
    // TODO Fix Zsh startup helper?

    iterm2::configure(&standard_user)?;
    login_items::configure(&standard_user)?;
    quicksilver::configure(&standard_user)?;
    hammerspoon::configure(&standard_user)?;
    karabiner::configure(&standard_user)?;
    network_link_conditioner::install()?;
    japicc::install()?;

    preferences::set(&standard_user)?;

    scripts::install(&standard_user)?;

    info!("Setup complete!");

    Ok(())
}
