use anyhow::{Result, anyhow};
use clap::{crate_authors, crate_description, crate_name};


fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

fn main() -> Result<()> {
    if !is_root() {
        return Err(anyhow!("This program must be run as root!"));
    }

    let app = clap::App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!());

    let _matches = app.get_matches();

    println!("Hello, world!");

    Ok(())
}
