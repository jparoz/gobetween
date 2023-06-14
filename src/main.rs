mod device;

use device::DeviceInfo;

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{builder::TypedValueParser as _, Parser};
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

/// Bounce MIDI commands between devices
#[derive(Parser, Debug)]
#[command(name = "gobetween", version, about, arg_required_else_help = true)]
struct Args {
    /// The path to the config file defining the devices in the system.
    config: PathBuf,

    /// Logging level
    #[arg(
        long,
        value_name = "LEVEL",
        default_value = "info",
        value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace", "off"])
                .map(|s| log::LevelFilter::from_str(&s).unwrap()),
        )]
    log: log::LevelFilter,
}

// @Todo: move this to its own module, maybe along with Args
#[derive(Deserialize, Debug)]
struct Config {
    devices: Vec<DeviceInfo>,
}

// @Todo: proper error handling
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Args::parse();

    // Set up logging
    env_logger::Builder::new()
        .filter(None, args.log)
        .format_timestamp(None)
        .init();

    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    // Connect to USB MIDI devices
    let mut midi_devices = Vec::new();
    for device_info in config.devices {
        log::info!("Connecting to device: {device_info:?}");
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
        log::trace!("Got a message: {msg:?}");
    }

    Ok(())
}
