use std::{collections::HashMap, str::FromStr};

use serde_with::{serde_as, DisplayFromStr};

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

/// Represents a specification for a number or range of numbers,
/// e.g. the velocity value of a MIDI note on message.
#[serde_as]
#[derive(serde::Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum Number {
    #[default]
    Any,
    Value(u32),
    Range(#[serde_as(as = "DisplayFromStr")] Range),
}

/// An inclusive range between two numbers,
/// parsed from e.g. `3-10`.
#[derive(Debug, Clone, Default)]
pub struct Range(pub u32, pub u32);

impl FromStr for Range {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split('-');
        let a = iter.next().ok_or("No `-`")?.trim();
        let b = iter.next().ok_or("No number after `-`")?.trim();
        if iter.next().is_none() {
            Ok(Self(
                a.parse()
                    .map_err(|_| "Couldn't parse number on left of `-`")?,
                b.parse()
                    .map_err(|_| "Couldn't parse number on right of `-`")?,
            ))
        } else {
            Err("Too many `-`s")
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't find device `{0}` (found in mapping)")]
    DeviceNotFound(String),
}
