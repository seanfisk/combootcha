use anyhow::Result;
use clap::Parser;
use nix::unistd::{fork, ForkResult};
use std::process::Command;

// An interesting project would be to rewrite this in Swift without the calls to pmset or caffeinate.
// pmset source: https://opensource.apple.com/source/PowerManagement/PowerManagement-703.1.5/pmset/pmset.c.auto.html
// caffeinate source: https://opensource.apple.com/source/PowerManagement/PowerManagement-703.1.5/caffeinate/caffeinate.c.auto.html

#[derive(Parser, Debug)]
#[command(
    author,
    about = "Run a command in the background while turning off the display.",
    trailing_var_arg = true,
)]
struct Args {
    #[arg(long, default_value_t = std::time::Duration::from_secs(5).into())]
    delay: humantime::Duration,

    #[arg(help = "Command to run", allow_hyphen_values=true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Create power assertion to avoid idle sleep
    // 2. Spawn the provided command
    // 3. Sleep for the delay
    // 4. Power off the display
    // 5. Wait for command to finish
    // 6. Sleep the system

    use ForkResult::*;
    match unsafe { fork() }.context("Fork failed")? {
        Parent { child } => {

            println!("Waiting for d")
        },
        Child => {
            std::thread::sleep(args.delay);
            Command::new("/usr/bin/pmset")
                .arg("displaysleepnow")
                .exec().context("Failed to exec pmset")
        }
    }
}



/*
PMSET = '/usr/bin/pmset'

    logging.basicConfig(format='[%(asctime)s] %(message)s', level=logging.INFO)

    logging.info('Putting display to sleep…')
    subprocess.run([PMSET, 'displaysleepnow'], check=True)
    command = ['/usr/bin/caffeinate'] + sys.argv[1:]
    logging.info('Running command: %r', command)
    proc = subprocess.run(command)
    logging.info('Putting machine to sleep…')
    # pmset prints 'Sleeping now...'; that is not useful in addition to our logging
subprocess.run([PMSET, 'sleepnow'], stdout=subprocess.DEVNULL)
    sys.exit(proc.returncode)
*/
