mod cathode;
mod default_browser;
mod env;
mod fs;
mod git;
mod hammerspoon;
mod hg;
mod homebrew;
mod iterm2;
mod japicc;
mod karabiner;
mod login_items;
mod login_shells;
mod network_link_conditioner;
mod path;
mod preferences;
mod quicksilver;
mod scripts;
mod ssh;
mod user;
mod user_defaults;
mod verbose_command;

use anyhow::{anyhow, Result};
use clap::{
    arg_enum, crate_authors, crate_description, crate_name, value_t, App, AppSettings::StrictUtf8,
    Arg,
};
use clap_logging::AppExt;
use log::{debug, info, LevelFilter};
use users::get_user_by_name;

arg_enum! {
    #[derive(Debug, Copy, Clone, PartialEq)]
    // We prefer to use case-sensitive names and want them to be all-lowercase. While it's possible to implement the enum ourselves, using clap::arg_enum is much easier. We simply have to put up with non-standard Rust naming, which is acceptable.
    #[allow(non_camel_case_types)]
    // TODO pub(crate) or hack?
    pub enum Config {
        personal,
        work,
    }
}

fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

fn get_standard_username(cli_value: Option<&str>) -> Result<String> {
    debug!("Looking for standard user from CLI");
    if let Some(v) = cli_value {
        debug!("Standard user set to {:?} from command line", v);
        Ok(v.to_owned())
    } else {
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

fn main() -> Result<()> {
    const STANDARD_USER_ARG_NAME: &str = "username";
    const HOMEBREW_ARG_NAME: &str = "homebrew";
    const BROWSER_ARG_NAME: &str = "set-default-browser";
    const CONFIG_ARG_NAME: &str = "config";

    if !is_root() {
        return Err(anyhow!("This program must be run as root!"));
    }

    let clap_logging_config = clap_logging::Config::new()?;

    let standard_user_arg = Arg::with_name(STANDARD_USER_ARG_NAME)
        .short("u")
        .long("standard-user")
        .help("Standard user to run as; defaults to value of SUDO_USER environment variable")
        .takes_value(true)
        .value_name("USERNAME");

    let homebrew_arg = Arg::with_name(HOMEBREW_ARG_NAME)
        .short("-H")
        .long(HOMEBREW_ARG_NAME)
        .help("Install Homebrew formulae and casks (takes a long time)");

    let browser_arg = Arg::with_name(BROWSER_ARG_NAME)
        .short("-B")
        .long(BROWSER_ARG_NAME)
        .help("Set the default browser (shows a prompt every time)");

    let config_arg = Arg::with_name(CONFIG_ARG_NAME)
        .index(1)
        .help("Configuration/purpose for this machine")
        .possible_values(&Config::variants());

    let app = App::new(crate_name!())
        .global_settings(&clap_logging_config.clap_settings())
        .global_setting(StrictUtf8)
        .about(crate_description!())
        .author(crate_authors!())
        .log_level_arg()
        .arg(standard_user_arg)
        .arg(homebrew_arg)
        .arg(browser_arg)
        .arg(config_arg);

    let matches = app.get_matches();

    clap_logging_config.init_logger(&matches, "SETUP_LOG_LEVEL", LevelFilter::Info)?;
    debug!("Logger was succesfully instantiated");

    let standard_username = get_standard_username(matches.value_of(STANDARD_USER_ARG_NAME))?;
    let standard_user = get_user_by_name(&standard_username).ok_or_else(|| {
        anyhow!(
            "User with name {:?} does not exist on this system!",
            standard_username
        )
    })?;

    let config = value_t!(matches.value_of(CONFIG_ARG_NAME), Config)?;

    if matches.is_present(HOMEBREW_ARG_NAME) {
        // homebrew::install_system(standard_user.clone())?;
        homebrew::install_deps(config, standard_user.clone())?;
    }

    login_shells::set(standard_user.clone())?;
    ssh::configure(config, &standard_user)?;
    // Note: Zsh interaction with path_helper was fixed, at least since Ventura

    iterm2::configure(&standard_user)?;
    login_items::configure(&standard_user)?;
    hammerspoon::configure(&standard_user)?;
    karabiner::configure(&standard_user)?;
    git::configure(standard_user.clone())?;
    cathode::install(standard_user.clone())?;

    match config {
        Config::personal => {
            // Trying to work without Quicksilver
            // quicksilver::configure(standard_user.clone())?;

            // TODO Do I need this?
            // hg::configure(standard_user.clone())?;
        }
        Config::work => {
            japicc::install()?;
        }
    }

    if matches.is_present(BROWSER_ARG_NAME) {
        default_browser::set(standard_user.clone())?;
    }

    preferences::set(&standard_user)?;

    scripts::install(&standard_user)?;

    info!("Setup complete!");

    Ok(())
}
