use ansi_term::Color::*;
use anyhow::Result;
use chrono::Local;
use clap::arg_enum;
use log::{error, Level, LevelFilter, Log, Metadata, Record};

use std::fmt::Display;
use std::io::{stderr, Write};
use std::str::FromStr;
use std::thread;

arg_enum! {
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum LogLevel {
        trace,
        debug,
        info,
        warn,
        error,
    }
}

// https://users.rust-lang.org/t/should-simple-enum-derive-copy-trait/11483/2
#[derive(Copy, Clone, Debug)]
pub(crate) enum ColorMode {
    Never,
    Auto,
    Always,
}

pub(crate) fn read_color_mode_from_env() -> Result<ColorMode> {
    if crate::env::get("CLICOLOR_FORCE")?.map_or(false, |value| value != "0") {
        // CLICOLOR_FORCE is present and non-zero
        return Ok(ColorMode::Always);
    }
    if crate::env::get("CLICOLOR")?.map_or(false, |value| value == "0") {
        // CLICOLOR is present and zero
        return Ok(ColorMode::Never);
    }
    Ok(ColorMode::Auto)
}

const LOGGING_ENV_VAR: &str = "LOG_LEVEL";
const DEFAULT_LOGGING_LEVEL: LevelFilter = LevelFilter::Info;

struct Logger {
    colorize: bool,
}

fn log_message<T: Display>(colorize: bool, level: Level, message: T) {
    let mut time_string = Local::now().format("%Y-%m-%dT%H:%M:%S%Z ").to_string();
    if colorize {
        time_string = Black.bold().paint(time_string).to_string();
    }

    let mut level_string = level.to_string();
    if colorize {
        level_string = match level {
            Level::Error => Red.paint(level_string),
            Level::Warn => Yellow.paint(level_string),
            Level::Info => Cyan.paint(level_string),
            Level::Debug => Blue.paint(level_string),
            Level::Trace => Purple.paint(level_string),
        }
        .to_string();
    }

    let current_thread = thread::current();
    let thread_name = current_thread.name().map_or_else(
        || format!("{:?}", current_thread.id()),
        |name| name.to_string(),
    );

    eprintln!(
        "{}[{}] {}: {}",
        time_string, thread_name, level_string, message
    );
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            log_message(self.colorize, record.level(), record.args());
        }
    }

    fn flush(&self) {
        if let Err(error) = stderr().flush() {
            error!("Failed to flush stderr: {}", error);
        }
    }
}

pub(crate) fn init(color_mode: ColorMode, level: Option<&str>) -> Result<()> {
    let colorize = match color_mode {
        ColorMode::Never => false,
        ColorMode::Always => true,
        ColorMode::Auto => atty::is(atty::Stream::Stderr),
    };

    let log_level = match level {
        Some(value) => {
            std::env::set_var(LOGGING_ENV_VAR, &value);
            LevelFilter::from_str(&value).unwrap()
        }
        None => match crate::env::get(LOGGING_ENV_VAR)? {
            Some(value) => match LevelFilter::from_str(&value) {
                Ok(value) => value,
                Err(_) => {
                    log_message(
                        colorize,
                        Level::Error,
                        format!(
                            "Invalid minimum log level {:?}; valid values are {:?}",
                            value,
                            &LogLevel::variants()
                        ),
                    );
                    std::process::exit(2);
                }
            },
            None => DEFAULT_LOGGING_LEVEL,
        },
    };

    std::env::set_var(LOGGING_ENV_VAR, log_level.to_string().to_ascii_lowercase());

    log::set_boxed_logger(Box::new(Logger { colorize: colorize }))
        .map(|()| log::set_max_level(log_level))?;

    Ok(())
}
