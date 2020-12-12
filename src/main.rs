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
enum Args {
    Cast {
        #[structopt(short = "r", default_value = "40")]
        rows: u16,
        #[structopt(short = "c", default_value = "80")]
        cols: u16,
    },
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Args::from_args();
    match opt {
        Args::Cast { rows, cols } => host::Host::new(cols, rows).await?.run().await?,
        _ => ()
    }
    Ok(())
}
