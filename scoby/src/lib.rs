mod cathode;
mod default_browser;
mod emacs;
mod env;
mod firefox;
pub mod fs;
mod git;
mod hammerspoon;
mod hg;
mod homebrew;
mod iterm2;
mod karabiner;
mod login_items;
mod login_shells;
mod network_link_conditioner;
pub mod path;
mod power_management;
mod preferences;
mod quicksilver;
mod rust;
mod scripts;
mod ssh;
pub mod user;
pub mod user_defaults;
pub mod verbose_command;
mod zsh;
mod text_buffer;

pub use path::Ext as PathExt;
pub use user::Ext as UserExt;

use anyhow::{anyhow, Result};
use clap::{AppSettings::StrictUtf8, Arg, ArgMatches};
use clap_logging::AppExt;
use log::{debug, LevelFilter};
use users::{get_user_by_name, User};

fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

pub fn check_root() -> Result<()> {
    if is_root() {
        Ok(())
    } else {
        Err(anyhow!("This program must be run as root!"))
    }
}

const STANDARD_USER_ARG_NAME: &str = "username";
const HOMEBREW_ARG_NAME: &str = "homebrew";
const BROWSER_ARG_NAME: &str = "set-default-browser";

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

// Return the username separately as we've already converted it to UTF-8
pub fn parse_standard_user(matches: &ArgMatches) -> Result<(String, User)> {
    let username = get_standard_username(matches.value_of(STANDARD_USER_ARG_NAME))?;
    let user = get_user_by_name(&username).ok_or_else(|| {
        anyhow!(
            "User with name {:?} does not exist on this system!",
            username
        )
    })?;
    Ok((username, user))
}

pub struct Scoby {
    clap_logging_config: clap_logging::Config,
    pub zsh: zsh::Config,
}

impl Scoby {
    pub fn new() -> Result<Self> {
        Ok(Self {
            clap_logging_config: clap_logging::Config::new()?,
            zsh: zsh::Config::new(),
        })
    }

    pub fn configure_cli<'a, 'b>(&self, app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
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

        app.global_settings(&self.clap_logging_config.clap_settings())
            .global_setting(StrictUtf8)
            .log_level_arg()
            .arg(standard_user_arg)
            .arg(homebrew_arg)
            .arg(browser_arg)
    }

    // I do not love this mega-function with a bunch of options. Going with it for now but registering the desire to improve it in the future.
    pub fn converge(
        self,
        matches: &ArgMatches,
        standard_user: User,
        brewfile_extra_bytes: Option<&[u8]>,
        ssh_config_extra_bytes: Option<&[u8]>,
        git_email: &str,
        hammerspoon_init_lua_extra_bytes: Option<&[u8]>,
    ) -> Result<()> {
        self.clap_logging_config
            .init_logger(matches, "COMBOOTCHA_LOG_LEVEL", LevelFilter::Info)?;
        debug!("Logger was successfully instantiated");

        // Run Homebrew first as it installs tools needed for later steps.
        // Yes, dependency installation can be disabled but we trust that the user will only disable it on subsequent runs.
        homebrew::configure()?;
        if matches.is_present(HOMEBREW_ARG_NAME) {
            homebrew::install_deps(standard_user.clone(), brewfile_extra_bytes)?;
        }

        // Command line tools
        login_shells::set(&standard_user)?;
        // Note: Zsh interaction with path_helper was fixed, at least since Ventura
        ssh::configure(&standard_user, ssh_config_extra_bytes)?;
        git::configure(git_email, standard_user.clone())?;
        scripts::install(&standard_user)?;
        self.zsh.converge(&standard_user)?;

        // Languages
        rust::configure(standard_user.clone())?;

        // Graphical programs
        iterm2::configure(&standard_user)?;
        emacs::configure(&standard_user)?;
        firefox::configure(&standard_user)?;
        cathode::install(standard_user.clone())?;
        hammerspoon::configure(&standard_user, hammerspoon_init_lua_extra_bytes)?;
        karabiner::configure(&standard_user)?;
        if matches.is_present(BROWSER_ARG_NAME) {
            default_browser::set(standard_user.clone())?;
        }

        // General preferences
        power_management::configure()?;
        login_items::configure(&standard_user)?;
        preferences::set(standard_user)?;

        Ok(())
    }
}
