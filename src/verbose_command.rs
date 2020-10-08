use anyhow::{anyhow, Context, Result};
use log::info;
use users::User;

use std::ffi::{OsStr, OsString};
use std::io::Write;
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
        self.log_command();
        use subprocess::{Popen, PopenConfig, Redirection};
        let mut argv = Vec::new();
        argv.push(self.program.clone());
        for arg in &self.args {
            argv.push(arg.clone());
        }
        // Note: Can't use Exec because it doesn't allow access to setuid, which we need
        let mut popen = Popen::create(
            &argv,
            PopenConfig {
                stdin: Redirection::None,
                stdout: Redirection::None,
                stderr: Redirection::None,
                cwd: self.cwd.as_ref().map(|p| p.as_os_str().to_owned()),
                setuid: self.user.as_ref().map(|u| u.uid()),
                ..Default::default()
            },
        )
        .context("Could not launch process")?; // TODO Improve context
                                               // TODO Check status
        popen.wait()?;
        Ok(())
        // let status = std_command
        //     .status()
        //     .with_context(|| make_context(&std_command))?;
        // check_status(&std_command, &status)
    }

    pub(crate) fn output(&self) -> Result<Vec<u8>> {
        self.log_command();
        use subprocess::{Popen, PopenConfig, Redirection};
        let mut argv = Vec::new();
        argv.push(self.program.clone());
        for arg in &self.args {
            argv.push(arg.clone());
        }
        // Note: Can't use Exec because it doesn't allow access to setuid, which we need
        let mut popen = Popen::create(
            &argv,
            PopenConfig {
                stdin: Redirection::None,
                stdout: Redirection::Pipe,
                stderr: Redirection::None,
                cwd: self.cwd.as_ref().map(|p| p.as_os_str().to_owned()),
                setuid: self.user.as_ref().map(|u| u.uid()),
                ..Default::default()
            },
        )
        .context("Could not launch process")?; // TODO Improve context
        let (stdout, _stderr) = popen.communicate_bytes(None)?;
        // TODO Check status
        Ok(stdout.ok_or_else(|| anyhow!("Stdout was not piped and therefore not captured"))?)
        // TODO Switch to using subprocess library so that the subprocess can just inherit stderr. We don't want to capture it in 99% of the cases.
        // let mut std_command = self.build_std_command();
        // let output = std_command
        //     .output()
        //     .with_context(|| make_context(&std_command))?;
        // std::io::stderr().write_all(&output.stderr)?;
        // check_status(&std_command, &output.status)?;
        // Ok(output.stdout)
    }

    fn log_command(&self) {
        // let mut std_command = std::process::Command::new(&self.program);
        // std_command.args(&self.args);
        // if let Some(cwd) = &self.cwd {
        //     std_command.current_dir(cwd);
        // }
        // if let Some(user) = &self.user {
        //     std_command.uid(user.uid());
        // }
        let mut argv = Vec::new();
        argv.push(self.program.clone());
        for arg in &self.args {
            argv.push(arg.clone());
        }
        info!(
            "=> {:?}{}{}",
            argv,
            self.cwd
                .as_ref()
                .map_or("".to_owned(), |d| format!(" (cwd: {:?})", d)),
            self.user
                .as_ref()
                .map_or("".to_owned(), |u| format!(" (user: {:?})", u.name()))
        );
    }
}

fn make_context(command: &std::process::Command) -> String {
    format!("Could not launch process {:?}", command)
}

// fn check_status(command: &std::process::Command, status: &ExitStatus) -> Result<()> {
//     if status.success() {
//         Ok(())
//     } else {
//         Err(anyhow!("Process {:?} failed with {}", command, status))
//     }
// }
