mod jettison;

use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    let (scoby_cli, app) = scoby::Cli::init()?;
    let matches = app.get_matches();
    let mut system_config = scoby_cli.parse_config(&matches)?;
    let standard_user = system_config.standard_user().clone();

    system_config.git().set_email("sean@seanfisk.com");

    system_config
        .homebrew()
        .add_global_brewfile_content(include_str!("Brewfile"));

    system_config.add_login_app("Jettison");

    system_config.converge(&matches)?;

    jettison::configure(&standard_user)?;

    info!("Setup complete!");

    Ok(())
}
