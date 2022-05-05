#[macro_use]
mod utils;
mod faderport;

use clap::Parser;
use std::net::IpAddr;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    sq_ip: IpAddr,

    #[clap(long)]
    fp_name: String,
}

fn main() {
    // Parse the command line arguments
    let args = Args::parse();

    // Start the tokio runtime
    let rt = Runtime::new().unwrap(); // @XXX: unwrap
    let _guard = rt.enter();

    let faderport = faderport::FaderPort::new(&args.fp_name);
    println!("faderport.is_ok(): {}", faderport.is_ok())
}
