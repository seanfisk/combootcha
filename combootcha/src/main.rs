use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    let (scoby_cli, app) = scoby::Cli::init()?;
    let matches = app.get_matches();
    let mut system_config = scoby_cli.parse_config(&matches)?;

    system_config.git().set_email("sean@seanfisk.com");

    system_config
        .homebrew()
        .add_global_brewfile_content(include_str!("Brewfile"));

    system_config.converge(&matches)?;

    info!("Setup complete!");

    Ok(())
}
