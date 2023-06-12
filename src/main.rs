mod midi;

use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

/// Bounce MIDI commands between devices
#[derive(Parser, Debug)]
#[command(name = "gobetween", version, about, arg_required_else_help = true)]
struct Args {
    /// The path to the config file defining the devices in the system.
    config: PathBuf,
}

// @Todo: move this to its own module, maybe along with Args
#[derive(Deserialize, Debug)]
struct Config {
    midi_devices: Vec<midi::DeviceInfo>,
}

// @Todo: proper error handling
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Args::parse();

    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    // Connect to USB MIDI devices
    let mut midi_devices = Vec::new();
    for device_info in config.midi_devices {
        println!("Connecting to device: {device_info:?}");
        midi_devices.push(device_info.connect()?);
    }

    // Print all broadcasted messages for debugging
    let mut streams = Vec::new();
    for device in midi_devices.iter() {
        let rx = device.subscribe();
        streams.push(BroadcastStream::new(rx));
    }

    let mut all_stream = futures::stream::select_all(streams);
    while let Some(msg) = all_stream.next().await {
        println!("Got a message: {msg:?}");
    }

    Ok(())
}
