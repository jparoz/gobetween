mod config;
mod device;
mod message;
mod midi;

use config::{Config, Mapping, Target};
use message::Transformer;

use std::{collections::HashMap, fs::File, path::PathBuf, str::FromStr};

use clap::{builder::TypedValueParser as _, Parser};
use tokio::task::JoinSet;
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

    log::trace!("Parsed config: {config:?}");

    if config.devices.is_empty() {
        log::warn!(
            "No devices specified in config file `{}`, exiting!",
            args.config.display()
        );
        return Ok(());
    }

    // Connect to the specified devices
    let mut devices = HashMap::new();
    let mut join_set = JoinSet::new();
    for device_info in config.devices {
        log::info!("Connecting to device: {device_info:?}");
        devices.insert(
            device_info.name.clone(),
            device_info.connect(&mut join_set)?,
        );
    }

    for (from_name, mappings) in config.mappings {
        let from_device = devices
            .get(&from_name)
            .ok_or_else(|| config::Error::DeviceNotFound(from_name.clone()))?;

        for Mapping {
            message_template: from_template,
            target:
                Target {
                    name: to_name,
                    message_template: to_template,
                    field_map,
                },
        } in mappings
        {
            let to_tx = devices
                .get(&to_name)
                .ok_or_else(|| config::Error::DeviceNotFound(to_name.clone()))?
                .tx
                .clone();
            let mut from_tx = from_device.subscribe();

            let transformer = Transformer {
                input: from_template,
                output: to_template,
                field_map,
            };

            join_set.spawn(async move {
                loop {
                    // @XXX: don't unwrap
                    let msg = from_tx.recv().await.unwrap();
                    if let Some(new_msg) = transformer.transform(msg) {
                        // @XXX: don't unwrap
                        to_tx.send(new_msg).await.unwrap();
                    }
                }
            });
        }
    }

    let mut streams = Vec::new();
    for device in devices.values() {
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
                    Ok(Ok(msg)) => log::info!("Task joined with message: {msg}"),

                    // Task joined properly, returning an Err
                    Ok(Err(err)) => log::error!("Task joined with error: {err}"),

                    // Task didn't join properly
                    Err(join_err) => log::error!("Join error: {join_err}"),
                }
            }

            else => { break }
        }
    }

    Ok(())
}
