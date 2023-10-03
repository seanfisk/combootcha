use anyhow::{anyhow, Context, Result};
use log::info;
use users::User;

use std::ffi::{OsStr, OsString};
use std::iter;
use std::path::{Path, PathBuf};

pub(crate) struct Command {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    user: Option<User>,
}

impl Command {
    pub(crate) fn new<S: AsRef<OsStr>>(program: S) -> Command {
        Command {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            cwd: None,
            user: None,
        }
    }

    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    pub(crate) fn cwd<P: AsRef<Path>>(&mut self, dir: P) -> &mut Command {
        self.cwd = Some(dir.as_ref().to_owned());
        self
    }

    pub(crate) fn user(&mut self, user: &User) -> &mut Command {
        self.user = Some(user.clone());
        self
    }

    pub(crate) fn run(&self) -> Result<()> {
        let mut popen = self.popen(false, false)?;
        self.wait(&mut popen)
    }

    pub(crate) fn run_with_input(&self, input: &[u8]) -> Result<()> {
        let mut popen = self.popen(true, false)?;
        popen.communicate_bytes(Some(input))?;
        self.wait(&mut popen)
    }

    pub(crate) fn output(&self) -> Result<Vec<u8>> {
        let mut popen = self.popen(false, true)?;
        let (stdout, _stderr) = popen.communicate_bytes(None)?;
        self.wait(&mut popen)?;
        Ok(stdout.ok_or_else(|| anyhow!("Stdout was not piped and therefore not captured"))?)
    }

    fn popen(&self, pipe_stdin: bool, pipe_stdout: bool) -> Result<subprocess::Popen> {
        info!("=> {}", self);

        // TODO I'm sure there is a more efficient way to do this
        let mut argv = Vec::new();
        argv.push(self.program.clone());
        for arg in &self.args {
            argv.push(arg.clone());
        }

        // Note: Can't use Exec because it doesn't allow access to setuid, which we need
        use subprocess::{Popen, PopenConfig, Redirection};
        Popen::create(
            &argv,
            PopenConfig {
                stdin: if pipe_stdin {
                    Redirection::Pipe
                } else {
                    Redirection::None
                },
                stdout: if pipe_stdout {
                    Redirection::Pipe
                } else {
                    Redirection::None
                },
                stderr: Redirection::None,
                cwd: self.cwd.as_ref().map(|p| p.as_os_str().to_owned()),
                setuid: self.user.as_ref().map(|u| u.uid()),
                ..Default::default()
            },
        )
        .with_context(|| format!("Could not launch process {}", self))
    }

    fn wait(&self, popen: &mut subprocess::Popen) -> Result<()> {
        let status = popen.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Process {} failed with {:?}", self, status))
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.debug_list()
            .entries(iter::once(&self.program).chain(&self.args))
            .finish()?;
        write!(
            f,
            "{}{}",
            self.cwd
                .as_ref()
                .map_or("".to_owned(), |d| format!(" (cwd: {:?})", d)),
            self.user
                .as_ref()
                .map_or("".to_owned(), |u| format!(" (user: {:?})", u.name()))
        )
    }
}
