use crate::verbose_command::Command;
use anyhow::Result;
use log::info;
use std::borrow::Cow;
use std::ffi::OsString;
use std::io::Write;
use users::{os::unix::UserExt, User};

use crate::UserExt as OtherUserExt;

pub struct Config {
    email: Option<Cow<'static, str>>,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self { email: None }
    }

    pub fn set_email<E: Into<Cow<'static, str>>>(&mut self, email: E) -> &mut Self {
        self.email = Some(email.into());
        self
    }

    pub(crate) fn converge(&self, standard_user: User) -> Result<()> {
        info!("Setting up personal Git preferences");

        standard_user.as_effective_user(|| {
            let git_config_dir = standard_user.home_dir().join(".config/git");
            crate::fs::ensure_dir(&git_config_dir)?;
            let gitignore_global_path = git_config_dir.join("ignore");
            info!("Writing {gitignore_global_path:?}");
            let mut file = crate::fs::create_file(&gitignore_global_path)?;
            let bytes = include_bytes!("ignore-global.txt");
            file.write_all(bytes)?;
            file.sync_all()?;
            Ok(())
        })?;

        let c = Gitconfig::new(standard_user.clone());
        {
            let s = c.section(&["user"]);
            s.string("name", "Sean Fisk")?;
            if let Some(ref email) = self.email {
                s.string("email", email.as_ref())?;
            }
        }
        c.section(&["alias"])
            // add all new and changed files in the repo, even if in a subdirectory
            // according to git-config(1), shell commands are executed from the top of the repository
            .string("all", "!git add --all")?
            .string("br", "branch")?
            .string("ci", "commit")?
            .string("cia", "commit --all")? // commit all tracked
            .string("ciam", "commit --all -m")? // commit all tracked with a message
            .string("cim", "commit -m")? // commit with a message
            .string("cl", "clone --recursive")? // note: we can't alias 'clone --recursive' to 'clone'; it just runs the original clone behavior
            .string("co", "checkout")?
            .string("cob", "checkout -b")? // create a new branch
            .string("pr", "remote prune origin")?
            .string("st", "status")?
            .string("mod", "ls-files --exclude-standard --modified --others")? // list modified or untracked files
            .string("sup", "!git submodule init && git submodule update")?
            .string("tags", "tag -n")? // show tags with their messages
            .string("nuke", "reset --hard HEAD")?;
        c.section(&["grep"])
            .bool("lineNumber", true)?
            .string("patternType", "perl")?;
        c.section(&["clean"]).bool("requireForce", false)?;
        c.section(&["push"])
            .string("default", "simple")?
            // https://stackoverflow.com/a/6089415
            .bool("autoSetupRemote", true)?;
        c.section(&["pull"]).bool("rebase", false)?;
        c.section(&["submodule"]).bool("recurse", true)?; // Automatically update submodules on 'git checkout'
        c.section(&["init"]).string("defaultBranch", "master")?;

        info!(
            "Setting up Git LFS for user with name {:?}",
            standard_user.name()
        );
        // All this does at this time of writing is to add the LFS filter to ~/.gitconfig
        git(standard_user)
            .args(&["lfs", "install"])
            // We shouldn't be in a repo when we run this, but be explicit that we don't want any repo setup
            .arg("--skip-repo")
            .run()
    }
}

struct Gitconfig {
    user: User,
}

impl<'a> Gitconfig {
    fn new(user: User) -> Gitconfig {
        Gitconfig { user }
    }

    fn section(&self, path: &'a [&'a str]) -> Section<'a> {
        Section::new(self.user.clone(), path)
    }
}

struct Section<'a> {
    path: &'a [&'a str],
    user: User,
}

impl<'a> Section<'a> {
    fn new(user: User, path: &'a [&'a str]) -> Section<'a> {
        Section { path, user }
    }

    fn string<V: Into<OsString>>(&self, key: &str, value: V) -> Result<&Section> {
        self.set(key, None, value)
    }

    fn bool(&self, key: &str, value: bool) -> Result<&Section> {
        self.set(key, Some("bool"), value.to_string())
    }

    fn set<V: Into<OsString>>(&self, key: &str, type_: Option<&str>, value: V) -> Result<&Section> {
        let dotted_path = self
            .path
            .iter()
            .chain(std::iter::once(&key))
            .copied()
            .collect::<Vec<_>>()
            .join(".");
        let mut command = git(self.user.clone());
        command.arg("config").arg("--global");
        if let Some(type_) = type_ {
            command.arg("--type").arg(type_);
        }
        command.arg("--").arg(dotted_path).arg(value).run()?;
        Ok(self)
    }
}

fn git(user: User) -> Command {
    let mut command = Command::new("/usr/local/bin/git"); // Always use the Homebrew version
    command.current_dir(user.home_dir()); // Running in a repo shouldn't be a problem, but let's not do it anyway
    command.user(user);
    command
}
