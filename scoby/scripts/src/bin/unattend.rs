use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    about = "Run a command in the background while turning off the display.",
    trailing_var_arg = true,
)]
struct Args {
    #[arg(long, default_value = "5s")]
    delay: Option<humantime::Duration>,

    #[arg(help = "Command to run", allow_hyphen_values=true)]
    command: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("delay: {:?}", args.delay);
    println!("command: {:?}", args.command);

    Ok(())
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
