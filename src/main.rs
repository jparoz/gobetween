mod faderport;

use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long)]
    sq_ip: IpAddr,

    #[clap(long)]
    fp_name: String,
}

fn main() {
    let args = Args::parse();
    println!("args: {:?}", args);

    let faderport = faderport::FaderPort::new(&args.fp_name);
    println!("faderport.is_ok(): {}", faderport.is_ok())
}
