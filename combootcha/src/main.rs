use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name};
use log::info;

fn main() -> Result<()> {
    scoby::check_root()?;

    let app = clap::App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!());
    let shared_setup = scoby::SharedSetup::new()?;
    let app = shared_setup.configure_cli(app);
    let matches = app.get_matches();
    let (_standard_username, standard_user) = scoby::parse_standard_user(&matches)?;

    shared_setup.run(
        &matches,
        standard_user.clone(),
        /*brewfile_extra_bytes=*/ Some(include_bytes!("Brewfile")),
        /*ssh_config_extra_bytes=*/ None,
        /*git_email=*/ "sean@seanfisk.com",
        /*zprofile_extra_bytes*/ None,
        /*zshrc_extra_bytes=*/ None,
        /*hammerspoon_init_lua_extra_bytes=*/ None,
    )?;

    info!("Setup complete!");

    Ok(())
}
