use clap::{ArgMatches, Arg};
use log::debug;
use anyhow::{anyhow, Result};
use users::{get_user_by_name, User};

const ARG_NAME: &str = "username";
pub(crate) fn arg<'a, 'b>() -> Arg<'a, 'b> {

Arg::with_name(ARG_NAME)
    .short("u")
    .long("standard-user")
    .help("Standard user to run as; defaults to value of SUDO_USER environment variable")
    .takes_value(true)
    .value_name("USERNAME")
}

fn parse_name(cli_value: Option<&str>) -> Result<String> {
    debug!("Looking for standard user from CLI");
    if let Some(v) = cli_value {
        debug!("Standard user set to {:?} from command line", v);
        Ok(v.to_owned())
    } else {
        debug!("Looking for standard user from SUDO_USER environment variable");
        match crate::env::get("SUDO_USER")? {
            Some(v) => {
                debug!("Standard user set to {:?} from SUDO_USER environment variable", v);
                Ok(v)
            }
            None => Err(anyhow!("Standard user not given by --standard-user command-line option nor SUDO_USER environment variable")),
        }
    }
}

// Return the username separately as we've already converted it to UTF-8
pub(crate) fn parse(matches: &ArgMatches) -> Result<(String, User)> {
    let name = parse_name(matches.value_of(ARG_NAME))?;
    let user = get_user_by_name(&name).ok_or_else(|| {
        anyhow!(
            "User with name {:?} does not exist on this system!",
            name
        )
    })?;
    Ok((name, user))
}
