fn is_root() -> bool {
    nix::unistd::Uid::current().is_root()
}

fn main() {
    if !is_root() {
        eprintln!("This program must be run as root!");
        std::process::exit(1);
    }
    println!("Hello, world!");
}
