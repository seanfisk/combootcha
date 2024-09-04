use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    let (scoby_cli, app) = scoby::Cli::init()?;
    let matches = app.get_matches();
    let mut global_config = scoby_cli.parse_config(&matches)?;

    global_config
        .homebrew()
        .add_global_brewfile_content(include_str!("Brewfile"));
    global_config.git().set_email("sean@seanfisk.com");

    info!("Setup complete!");

    Ok(())
}
