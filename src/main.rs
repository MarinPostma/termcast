#[macro_use]
extern crate bitflags;

mod backends;
mod cell;
mod host;
mod layout;
mod style;
mod terminal;

use structopt::StructOpt;
use anyhow::Result;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    #[structopt(flatten)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Cast {
        #[structopt(short = "r", default_value = "40")]
        rows: usize,
        #[structopt(short = "c", default_value = "80")]
        cols: usize,
    },
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Options::from_args();
    if opt.debug {
        env_logger::init();
    }
    match opt.command {
        Command::Cast { rows, cols } => host::Host::new(cols, rows).await?.run().await?,
        _ => ()
    }
    Ok(())
}
