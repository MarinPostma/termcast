#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

mod terminal;
mod style;
mod backends;
mod layout;
mod cell;
mod host;

#[derive(Default)]
struct State;

use structopt::StructOpt;


#[derive(StructOpt)]
enum Args {
    Cast {
        #[structopt(short = "r", default_value = "40")]
        rows: u16,
        #[structopt(short = "c", default_value = "80")]
        cols: u16,
    },
    Watch {
        addr: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Args::from_args();
    match opt {
        Args::Cast { rows, cols } => {
            host::init_host(cols, rows).await?;
        }
        _ => ()
    }
    Ok(())
}
