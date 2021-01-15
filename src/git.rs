use crate::verbose_command::Command;
use anyhow::Result;
use log::info;
use users::{os::unix::UserExt, User};

pub(crate) fn configure(standard_user: &User) -> Result<()> {
    info!("Setting up personal Git preferences");
    let c = Gitconfig::new(&standard_user);
    c.section(&["user"])
        .string("name", "Sean Fisk")?
        .string("email", "sean@seanfisk.com")?;
    c.section(&["core"])
        .string("excludesfile", "~/.gitignore-global")?;
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
    c.section(&["push"]).string("default", "simple")?;
    c.section(&["pull"]).bool("rebase", false)?;
    c.section(&["submodule"]).bool("recurse", true)?; // Automatically update submodules on 'git checkout'
    c.section(&["init"]).string("defaultBranch", "master")?;

    info!(
        "Setting up Git LFS for user with name {:?}",
        standard_user.name()
    );
    // All this does at this time of writing is to add the LFS filter to ~/.gitconfig
    Command::new("git")
        .args(&["lfs", "install"])
        // We shouldn't be in a repo when we run this, but be explicit that we don't want any repo setup
        .arg("--skip-repo")
        .user(&standard_user)
        .run()
}

struct Gitconfig<'a> {
    user: &'a User,
}

impl<'a> Gitconfig<'a> {
    fn new(user: &'a User) -> Gitconfig {
        Gitconfig { user: user }
    }

    fn section(&self, path: &'a [&'a str]) -> Section<'a> {
        Section::new(&self.user, &path)
    }
}

struct Section<'a> {
    path: &'a [&'a str],
    user: &'a User,
}

impl<'a> Section<'a> {
    fn new(user: &'a User, path: &'a [&'a str]) -> Section<'a> {
        Section {
            path: path,
            user: user,
        }
    }

    fn string(&self, key: &str, value: &str) -> Result<&Section> {
        self.set(key, None, value)
    }

    fn bool(&self, key: &str, value: bool) -> Result<&Section> {
        self.set(key, Some("bool"), &value.to_string())
    }

    fn set(&self, key: &str, type_: Option<&str>, value: &str) -> Result<&Section> {
        let dotted_path = self
            .path
            .iter()
            .chain(std::iter::once(&key))
            .cloned()
            .collect::<Vec<_>>()
            .join(".");
        let mut command = Command::new("git");
        command.arg("config").arg("--global");
        if let Some(type_) = type_ {
            command.arg("--type").arg(type_);
        }
        command
            .arg("--")
            .arg(dotted_path)
            .arg(value)
            .cwd(self.user.home_dir()) // Running in a repo shouldn't be a problem, but let's not do it anyway
            .user(self.user)
            .run()?;
        Ok(self)
    }
}
