use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    about = "Reverse-resolve an IP address on macOS using the native DNS facilities."
)]
struct Args {
    #[arg(help = "IP address to reverse-resolve")]
    ip_address: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    scripts::dscacheutil(
        "host",
        "ip_address",
        &args.ip_address,
        "reverse-resolve IP address",
    )
}
