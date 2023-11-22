use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    about = "Resolve a domain name on macOS using the native DNS facilities."
)]
struct Args {
    #[arg(help = "Domain name to resolve")]
    domain_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    scripts::dscacheutil("host", "name", &args.domain_name, "resolve domain name")
}
