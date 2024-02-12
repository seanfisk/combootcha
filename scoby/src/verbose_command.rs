use anyhow::{anyhow, Context, Result};
use log::info;
use users::User;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::iter;
use std::path::PathBuf;

// We are using the subprocess crate instead of std::process::Command because Command is vulnerable
// to deadlock when writing to stdin:
// http://doc.rust-lang.org/1.73.0/std/process/struct.Stdio.html#method.piped
//
// However, I'm not sure of the maintenance status of subprocess so we may want to switch to
// std::process::Command eventually. The deadlock can be overcome with careful programming. In fact,
// we can consult subprocess for how to avoid this!
//
// Advice on creating builders:
// https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
// https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
//
// The way that std::process::Command manages to implement a non-consuming builder is by converting
// borrowed OS strings to owned C strings when each builder method is called. For example:
//
// https://github.com/rust-lang/rust/blob/1.72.1/library/std/src/sys/unix/process/process_common.rs#L248-L250
//
// Since we're wrapping another library, we don't have that luxury.

pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    user: Option<User>,
    // Use a tree map for a consistent ordering. We shouldn't be relying on an explicit order or duplicates for environment variables (although both are technically possible).
    env_overrides: BTreeMap<OsString, OsString>,
}

impl Command {
    pub fn new<S: Into<OsString>>(program: S) -> Command {
        Command {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            user: None,
            env_overrides: BTreeMap::new(),
        }
    }

    pub fn arg<S: Into<OsString>>(&mut self, arg: S) -> &mut Command {
        self.args.push(arg.into());
        self
    }

    pub fn args<I>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator,
        I::Item: Into<OsString>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    pub fn current_dir<P: Into<PathBuf>>(&mut self, path: P) -> &mut Command {
        self.current_dir = Some(path.into());
        self
    }

    // I am interested in the possibility of accepting &User instead of User, but it's of low importance
    pub fn user(&mut self, user: User) -> &mut Command {
        self.user = Some(user);
        self
    }

    /// Works like https://doc.rust-lang.org/std/process/struct.Command.html#method.env
    pub fn env<N: Into<OsString>, V: Into<OsString>>(&mut self, name: N, value: V) -> &mut Command {
        self.env_overrides.insert(name.into(), value.into());
        self
    }

    pub fn run(&self) -> Result<()> {
        let mut popen = self.popen(false, false)?;
        self.wait(&mut popen)
    }

    pub fn run_with_input(&self, input: &[u8]) -> Result<()> {
        let mut popen = self.popen(true, false)?;
        popen.communicate_bytes(Some(input))?;
        self.wait(&mut popen)
    }

    pub fn output(&self) -> Result<Vec<u8>> {
        let mut popen = self.popen(false, true)?;
        let (stdout, _stderr) = popen.communicate_bytes(None)?;
        self.wait(&mut popen)?;
        stdout.ok_or_else(|| anyhow!("Stdout was not piped and therefore not captured"))
    }

    fn popen(&self, pipe_stdin: bool, pipe_stdout: bool) -> Result<subprocess::Popen> {
        use subprocess::{Popen, PopenConfig, Redirection};

        info!("=> {}", self);

        let mut argv = Vec::new();
        argv.push(&self.program);
        for arg in &self.args {
            argv.push(arg);
        }

        // From subprocess documentation:
        //
        //     Environment variables to pass to the subprocess.
        //
        //     If this is None, environment variables are inherited from the calling process. Otherwise, the specified variables are used instead.
        //
        //     Duplicates are eliminated, with the value taken from the variable appearing later in the vector.
        //
        // Source: https://docs.rs/subprocess/latest/subprocess/struct.PopenConfig.html#structfield.env
        //
        // However, our builder works by providing environment variable overrides, just like std::process::Command. Massage our overrides to the format expected by PopenConfig.
        let env = if self.env_overrides.is_empty() {
            None
        } else {
            let mut list = Vec::new();
            for pair in std::env::vars_os() {
                list.push(pair);
            }
            for (name, value) in &self.env_overrides {
                list.push((name.clone(), value.clone()));
            }
            Some(list)
        };

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
                env,
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
        if !self.env_overrides.is_empty() {
            write!(f, " (env: {:?})", self.env_overrides)?;
        }
        Ok(())
    }
}
