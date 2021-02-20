use quicli::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
}

fn main() {
    let args = Cli::from_args();
}
