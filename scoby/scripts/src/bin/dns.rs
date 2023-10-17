use anyhow::Result;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author)]
struct Args {
    domain_name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("domain name: {}", args.domain_name);
    Ok(())
}

/*
#!/Users/Fisk/.local/bin/python
# -*- mode: python; -*-

# Python 2/3 compatible

from __future__ import (
division, absolute_import, print_function, unicode_literals)
import sys
import argparse
import subprocess

arg_parser = argparse.ArgumentParser(description=(
"Resolve a domain name on macOS using the native DNS facilities."))
arg_parser.add_argument('domain_name', help='Domain name to resolve')
args = arg_parser.parse_args()

# dscacheutil will print no output and exit with a 0 code if it doesn't find
# the specified name. This is really unclear in the context of using it for DNS
# lookups, so we are overriding it.

output = subprocess.check_output(
[
'/usr/bin/dscacheutil',
'-q', 'host',
'-a', 'name', args.domain_name,
    ],
    universal_newlines=True
)

if len(output) == 0:
    sys.exit('Could not resolve domain name: {}'.format(args.domain_name))
else:
    print(output, end='')
*/
