mod device;

use device::DeviceInfo;
use tokio::task::JoinSet;

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

    let config_file = File::open(&args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    if config.devices.is_empty() {
        log::warn!(
            "No devices specified in config file `{}`, exiting!",
            args.config.display()
        );
        return Ok(());
    }

    // Connect to the specified devices
    let mut devices = Vec::new();
    let mut join_set = JoinSet::new();
    for device_info in config.devices {
        log::info!("Connecting to device: {device_info:?}");
        devices.push(device_info.connect(&mut join_set)?);
    }

    let mut streams = Vec::new();
    for device in devices.iter() {
        let rx = device.subscribe();
        streams.push(BroadcastStream::new(rx));
    }
    let mut message_echo_stream = futures::stream::select_all(streams);

    loop {
        tokio::select! {
            // Print all broadcasted messages for debugging
            Some(msg) = message_echo_stream.next() => {
                log::trace!("Got a message: {msg:?}");
            }

            // Join all the spawned tasks,
            // so that we can (in principle) do something with the return values.
            Some(join_result) = join_set.join_next() => {
                match join_result {
                    // Task joined properly, returning the happy-path message for that device
                    Ok(Ok(msg)) => log::info!("{msg}"),

                    // Task joined properly, returning an Err
                    Ok(Err(err)) => log::error!("{err}"),

                    // Task didn't join properly
                    Err(join_err) => log::error!("Join error: {join_err}"),
                }
            }

            else => { break }
        }
    }

    Ok(())
}
