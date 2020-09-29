use anyhow::{anyhow, Context, Result};
use log::info;
use users::User;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

pub(crate) struct Command {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    uid: Option<u32>,
}

impl Command {
    pub(crate) fn new<S: AsRef<OsStr>>(program: S) -> Command {
        Command {
            program: program.as_ref().to_owned(),
            args: Vec::new(),
            cwd: None,
            uid: None,
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
        self.uid = Some(user.uid());
        self
    }

    pub(crate) fn run(&self) -> Result<()> {
        let mut std_command = self.build_std_command();
        let status = std_command
            .status()
            .with_context(|| format!("Could not launch process {:?}", std_command))?;
        check_status(&std_command, &status)
    }

    pub(crate) fn output(&self) -> Result<Vec<u8>> {
        // TODO Switch to using subprocess library so that the subprocess can just inherit stderr. We don't want to capture it in 99% of the cases.
        let mut std_command = self.build_std_command();
        let output = std_command
            .output()
            .with_context(|| format!("Could not launch process {:?}", std_command))?;
        std::io::stderr().write_all(&output.stderr)?;
        check_status(&std_command, &output.status)?;
        Ok(output.stdout)
    }

    fn build_std_command(&self) -> std::process::Command {
        let mut std_command = std::process::Command::new(&self.program);
        std_command.args(&self.args);
        if let Some(cwd) = &self.cwd {
            std_command.current_dir(cwd);
        }
        if let Some(uid) = self.uid {
            std_command.uid(uid);
        }
        info!(
            "=> {:?}{}",
            std_command,
            self.cwd
                .as_ref()
                .map_or("".to_owned(), |d| format!(" (cwd: {:?})", d))
        );
        std_command
    }
}

fn check_status(command: &std::process::Command, status: &ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Process {:?} failed with {}", command, status))
    }
}
