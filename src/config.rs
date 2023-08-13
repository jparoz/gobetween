use std::collections::HashMap;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub devices: Vec<DeviceInfo>,
    pub mappings: HashMap<String, Vec<Mapping>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeviceInfo {
    /// The name of the device. Can be anything.
    pub name: String,

    #[serde(flatten)]
    pub connection_info: ConnectionInfo,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum ConnectionInfo {
    /// MIDI over TCP connection information.
    TcpMidi {
        /// The address and port of the TCP MIDI device.
        /// This will be used by [`ToSocketAddrs`](std::net::ToSocketAddrs),
        /// so should be something like `"123.456.40.13:8033"`.
        midi_address: String,
    },

    /// MIDI connection information.
    Midi {
        /// The name of the MIDI input device.
        midi_in: String,
        /// The name of the MIDI output device.
        midi_out: String,
    },
}

// #[derive(serde::Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum MessageTemplate {
//     Midi(midi::MessageTemplate),
// }
pub use crate::midi::MessageTemplate;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Mapping {
    #[serde(rename = "from")]
    pub message_template: MessageTemplate,

    #[serde(rename = "to")]
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Target {
    #[serde(rename = "target")]
    pub name: String,

    #[serde(rename = "mapping")]
    #[serde(default)]
    pub field_map: HashMap<String, String>,

    #[serde(flatten)]
    pub message_template: MessageTemplate,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't find device `{0}` (found in mapping)")]
    DeviceNotFound(String),
}
