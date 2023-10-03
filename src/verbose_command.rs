use anyhow::{anyhow, Context, Result};
use log::info;
use users::User;

use std::ffi::OsString;
use std::iter;
use std::path::PathBuf;

pub(crate) struct Command {
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    user: Option<User>,
}

impl Command {
    pub(crate) fn new<S: Into<OsString>>(program: S) -> Command {
        Command {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            user: None,
        }
    }

    pub(crate) fn arg<S: Into<OsString>>(&mut self, arg: S) -> &mut Command {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn args<I>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator,
        I::Item: Into<OsString>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    pub(crate) fn current_dir<P: Into<PathBuf>>(&mut self, path: P) -> &mut Command {
        self.current_dir = Some(path.into());
        self
    }

    pub(crate) fn user(&mut self, user: &User) -> &mut Command {
        // TODO Don't accept a ref just to clone it
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
        stdout.ok_or_else(|| anyhow!("Stdout was not piped and therefore not captured"))
    }

    fn popen(&self, pipe_stdin: bool, pipe_stdout: bool) -> Result<subprocess::Popen> {
        use subprocess::{Popen, PopenConfig, Redirection};

        info!("=> {}", self);

        // TODO I'm sure there is a more efficient way to do this
        let mut argv = Vec::new();
        argv.push(&self.program);
        for arg in &self.args {
            argv.push(arg);
        }

        // Note: Can't use Exec because it doesn't allow access to setuid, which we need
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
                cwd: self.current_dir.as_ref().map(|p| p.as_os_str().to_owned()),
                setuid: self.user.as_ref().map(User::uid),
                ..Default::default()
            },
        )
        .with_context(|| format!("Could not launch process {self}"))
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
        if let Some(dir) = &self.current_dir {
            write!(f, " (cwd: {dir:?})")?;
        }
        if let Some(user) = &self.user {
            write!(f, " (user: {:?})", user.name())?;
        }
        Ok(())
    }
}
