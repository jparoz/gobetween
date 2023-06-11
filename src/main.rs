mod midi;

use std::fs::File;
use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use tokio::runtime::Runtime;

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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command line arguments
    let args = Args::parse();

    let config_file = File::open(args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    // Start the tokio runtime
    let rt = Runtime::new()?;
    let _guard = rt.enter();

    // Connect to USB MIDI devices
    let mut midi_devices = Vec::new();
    for device_info in config.midi_devices {
        println!("Connecting to device: {device_info:?}");
        midi_devices.push(device_info.connect()?);
    }

    // // FaderPort test code
    // let mut faderport = midi::UsbDevice::new(&config.usb_midi_devices[0])?;
    // {
    //     let mut rx = faderport.subscribe();

    //     tokio::spawn(async move {
    //         loop {
    //             let received = rx.recv().await;
    //             println!("{:?}", received);
    //         }
    //     });

    //     std::thread::sleep(std::time::Duration::new(2, 0));

    //     faderport.update(faderport::message::Message::Led(
    //         faderport::message::Led::Write,
    //         faderport::message::LedState::Off,
    //     ))?;
    // }

    // // SQ test code
    // let sq = midi::TcpDevice::new(&config.tcp_midi_devices[0])?;
    // {
    //     let mut rx = sq.subscribe();

    //     tokio::spawn(async move {
    //         loop {
    //             let received = rx.recv().await;
    //             println!("{:?}", received);
    //         }
    //     });
    // }

    // Quit the program when enter is pressed
    let _ = std::io::stdin().read_line(&mut String::new());

    Ok(())
}
