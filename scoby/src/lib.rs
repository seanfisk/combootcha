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
mod standard_user;
mod text_buffer;
pub mod user;
pub mod user_defaults;
pub mod verbose_command;
mod zsh;

pub use path::Ext as PathExt;
pub use user::Ext as UserExt;

use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_description, AppSettings::StrictUtf8, ArgMatches};
use clap_logging::AppExt;
use log::{debug, LevelFilter};
use users::User;

fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

fn check_root() -> Result<()> {
    if is_root() {
        Ok(())
    } else {
        Err(anyhow!("This program must be run as root!"))
    }
}

pub struct Cli {
    clap_logging_config: clap_logging::Config,
}

impl Cli {
    pub fn init<'a, 'b>() -> Result<(Self, clap::App<'a, 'b>)> {
        check_root()?;

        let clap_logging_config = clap_logging::Config::new()?;

        let app = clap::App::new("combootcha")
            .about(crate_description!())
            .author(crate_authors!())
            .global_settings(&clap_logging_config.clap_settings())
            .global_setting(StrictUtf8)
            .log_level_arg()
            .arg(standard_user::arg())
            .arg(homebrew::arg())
            .arg(default_browser::arg());

        Ok((
            Self {
                clap_logging_config,
            },
            app,
        ))
    }

    pub fn parse_config(&self, matches: &ArgMatches) -> Result<GlobalConfig> {
        self.clap_logging_config
            .init_logger(matches, "COMBOOTCHA_LOG_LEVEL", LevelFilter::Info)?;

        let (standard_username, standard_user) = standard_user::parse(matches)?;
        let zsh = zsh::Config::new();
        let homebrew = homebrew::Config::new(matches);
        let ssh = ssh::Config::new();
        let git = git::Config::new();
        let hammerspoon = hammerspoon::Config::new();

        Ok(GlobalConfig {
            standard_username,
            standard_user,
            zsh,
            homebrew,
            ssh,
            git,
            hammerspoon,
        })
    }
}

/// Configuration of the operating system and everything on it.
pub struct GlobalConfig {
    // Making these fields public allows a user to instantiate this struct, which is not desirable. I want it to be an opaque type. So generate getters.
    pub(crate) standard_username: String,
    pub(crate) standard_user: User,
    pub(crate) zsh: zsh::Config,
    pub(crate) homebrew: homebrew::Config,
    pub(crate) ssh: ssh::Config,
    pub(crate) git: git::Config,
    pub(crate) hammerspoon: hammerspoon::Config,
}

impl GlobalConfig {
    pub fn standard_username(&self) -> &str {
        &self.standard_username
    }

    pub fn standard_user(&self) -> &User {
        &self.standard_user
    }

    pub fn zsh(&mut self) -> &mut zsh::Config {
        &mut self.zsh
    }

    pub fn homebrew(&mut self) -> &mut homebrew::Config {
        &mut self.homebrew
    }

    pub fn ssh(&mut self) -> &mut ssh::Config {
        &mut self.ssh
    }

    pub fn git(&mut self) -> &mut git::Config {
        &mut self.git
    }

    pub fn hammerspoon(&mut self) -> &mut hammerspoon::Config {
        &mut self.hammerspoon
    }

    // Some TextBuffers have additional data written to them and I don't want to have to copy-on-write. Is consuming self the best practice here? Not sure, but it solves the issue neatly. I don't see a need to converge multiple times per Combootcha invocation.
    pub fn converge(self, matches: &ArgMatches) -> Result<()> {
        debug!("Logger was successfully instantiated");

        // Run Homebrew first as it installs tools needed for later steps.
        // Yes, dependency installation can be disabled but we trust that the user will only disable it on subsequent runs.
        self.homebrew.converge(self.standard_user.clone())?;
        // Command line tools
        login_shells::set(&self.standard_user)?;
        // Note: Zsh interaction with path_helper was fixed, at least since Ventura
        self.ssh.converge(&self.standard_user)?;
        self.git.converge(self.standard_user.clone())?;
        scripts::install(&self.standard_user)?;
        self.zsh.converge(&self.standard_user)?;

        // Languages
        rust::configure(self.standard_user.clone())?;

        // Graphical programs
        iterm2::configure(&self.standard_user)?;
        emacs::configure(&self.standard_user)?;
        firefox::configure(&self.standard_user)?;
        cathode::install(self.standard_user.clone())?;
        self.hammerspoon.converge(&self.standard_user)?;
        karabiner::configure(&self.standard_user)?;
        default_browser::converge(matches, &self.standard_user)?;

        // General preferences
        power_management::configure()?;
        login_items::configure(&self.standard_user)?;
        preferences::set(self.standard_user)?;

        Ok(())
    }
}
