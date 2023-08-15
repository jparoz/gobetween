use std::io;

use midly::live::LiveEvent;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinSet,
};

use crate::{
    config::{ConnectionInfo, DeviceInfo},
    midi,
};

impl DeviceInfo {
    // @Todo @Cleanup: this shouldn't have LiveEvent in its return type
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

    /// This is the sender to which we send messages for this device.
    pub tx: mpsc::Sender<Message>,

    /// This is a clone of the sender moved to the main device callback.
    /// Use [`Device::subscribe`] to receive messages from this device.
    pub broadcast_tx: broadcast::Sender<Message>,
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
