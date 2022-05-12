#[macro_use]
mod utils;
mod faderport;
mod sq;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Args::parse();

    // Start the tokio runtime
    let rt = Runtime::new().unwrap(); // @XXX: unwrap
    let _guard = rt.enter();

    // FaderPort test code
    let faderport = faderport::FaderPort::new(&args.fp_name)?;

    let mut rx = faderport.subscribe();

    tokio::spawn(async move {
        loop {
            let received = rx.recv().await;
            println!("{:?}", received);
        }
    });

    // SQ test code
    let sq = sq::SQ::new(args.sq_ip)?;

    // Quit the program when enter is pressed
    let _ = std::io::stdin().read_line(&mut String::new());

    Ok(())
}
