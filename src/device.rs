use std::io;

use midly::live::LiveEvent;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

use crate::midi;

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
    pub fn connect(
        &self,
        join_set: &mut JoinSet<Result<String, Error>>,
    ) -> Result<Device<LiveEvent<'static>>, Error> {
        use ConnectionInfo::*;
        match &self.connection_info {
            TcpMidi { midi_address } => {
                Device::tcp_midi(join_set, &self.name, midi_address.to_string())
            }
            Midi { midi_in, midi_out } => Device::midi(join_set, &self.name, midi_in, midi_out),
        }
    }
}

pub struct Device<Message> {
    /// The name of the device. Can be anything.
    // @Todo: This could probably be a reference into the originating DeviceInfo
    pub name: String,

    /// This is the sender to which we send MIDI devices for this device.
    pub tx: mpsc::Sender<Message>,

    /// This is a clone of the sender moved to the main device callback.
    /// Use [`Device::subscribe`] to receive MIDI messages from this device.
    pub broadcast_tx: broadcast::Sender<Message>,

    /// All of the mappings where this device is the "from".
    // @Todo: this should hold JoinHandles from the spawned tokio threads (?? maybe not needed)
    pub mapped: Vec<()>,
}

impl<Message> Device<Message> {
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.broadcast_tx.subscribe()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("MIDI device error: {0}")]
    Midi(#[from] midi::device::Error),
}
