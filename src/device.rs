use std::io;

use midi_msg::MidiMsg;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

use crate::mapping::{FieldMap, Mapped};
use crate::midi;
use crate::spec::Spec;

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

impl DeviceInfo {
    pub fn connect(&self, join_set: &mut JoinSet<Result<String, Error>>) -> Result<Device, Error> {
        use ConnectionInfo::*;
        match &self.connection_info {
            TcpMidi { midi_address } => {
                Device::tcp_midi(join_set, &self.name, midi_address.to_string())
            }
            Midi { midi_in, midi_out } => Device::midi(join_set, &self.name, midi_in, midi_out),
        }
    }
}

// @Todo: parameterise over the message type, e.g. Device<MidiMsg>
pub struct Device {
    /// The name of the device. Can be anything.
    // @Todo: This could probably be a reference into the originating DeviceInfo
    pub name: String,

    /// This is the sender to which we send MIDI devices for this device.
    pub tx: mpsc::Sender<MidiMsg>,

    /// This is a clone of the sender moved to the main device callback.
    /// Use [`Device::subscribe`] to receive MIDI messages from this device.
    pub broadcast_tx: broadcast::Sender<MidiMsg>,

    /// All of the mappings where this device is the "from".
    pub mapped: Vec<Mapped<MidiMsg>>,
}

impl Device {
    pub fn subscribe(&self) -> broadcast::Receiver<MidiMsg> {
        self.broadcast_tx.subscribe()
    }

    pub fn map_to(
        &mut self,
        tx: mpsc::Sender<MidiMsg>,
        trigger: Spec,
        target: Spec,
        field_map: FieldMap,
    ) {
        self.mapped
            .push(Mapped::new(trigger, target, tx, field_map));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("MIDI device error: {0}")]
    Midi(#[from] midi::device::Error),
}
