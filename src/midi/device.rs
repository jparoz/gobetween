use std::io;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{broadcast, mpsc};

use midi_msg::{Channel, ChannelVoiceMsg, ControlChange, MidiMsg};

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
    /// TCP MIDI connection information.
    Tcp {
        /// This will be used by [`ToSocketAddrs`],
        /// so should be something like `"123.456.40.13:8033"`.
        address: String,
    },

    /// USB MIDI connection information.
    Usb {
        /// The name of the USB MIDI input device.
        midi_in: String,
        /// The name of the USB MIDI output device.
        midi_out: String,
    },
}

impl DeviceInfo {
    pub fn connect(&self) -> Result<Device, Error> {
        use ConnectionInfo::*;
        match &self.connection_info {
            Tcp { address } => Device::tcp(&self.name, address.to_string()),
            Usb { midi_in, midi_out } => Device::usb(&self.name, midi_in, midi_out),
        }
    }
}

#[derive(Debug)]
pub struct Device {
    /// The name of the device. Can be anything.
    // @Todo: This could probably be a reference into the originating DeviceInfo
    pub name: String,

    /// This is a clone of the sender moved to the main device callback.
    /// Use [`TcpDevice::subscribe`] to receive MIDI messages from this device.
    broadcast_tx: broadcast::Sender<MidiMsg>,

    /// This is the sender to which we send MIDI devices for this device.
    tx: mpsc::Sender<MidiMsg>,
}

impl Device {
    pub fn subscribe(&self) -> broadcast::Receiver<MidiMsg> {
        self.broadcast_tx.subscribe()
    }

    pub async fn send(&mut self, msg: MidiMsg) -> Result<(), Error> {
        self.tx.send(msg).await?;
        Ok(())
    }

    fn tcp(name: &str, addr: String) -> Result<Self, Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        tokio::spawn(async move {
            // @Fixme: shouldn't unwrap
            let mut socket = TcpStream::connect(addr).await.unwrap();
            let mut buf = BytesMut::new();
            let broadcast_tx = cloned_broadcast_tx;
            let mut ctx = midi_msg::ReceiverContext::new();

            loop {
                select! {
                    bytes_read = socket.read_buf(&mut buf) => {
                        // @Fixme: shouldn't unwrap
                        let _bytes_read = bytes_read.unwrap();

                        // Keep track of the chunks we split off
                        let mut splits = Vec::new();

                        // @Todo @XXX: don't ignore this error, we could get stuck
                        while let Ok((msg, len)) =
                            MidiMsg::from_midi_with_context(&buf, &mut ctx) {
                            splits.push(buf.split_to(len)); // @Checkme
                            // @Fixme: shouldn't unwrap
                            broadcast_tx.send(msg).unwrap();
                        }

                        if !buf.is_empty()  {
                            todo!("handle partial messages in TCP packets")
                        }

                        // Unsplit all the chunks to recover the whole buffer
                        while let Some(mut chunk) = splits.pop() {
                            chunk.unsplit(buf);
                            buf = chunk;
                        }
                    }
                    msg = rx.recv() => {
                        let msg = msg.unwrap(); // @Fixme: shouldn't unwrap, maybe pattern match?
                        socket.write_all(&msg.to_midi()).await.unwrap(); // @Fixme: shouldn't unwrap
                    }
                }
            }
        });

        Ok(Device {
            name: name.to_string(),
            broadcast_tx,
            tx,
        })
    }

    fn usb(name: &str, in_name: &str, out_name: &str) -> Result<Self, Error> {
        todo!()
    }
}

// @Todo: proper errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Channel send error, couldn't send MIDI message {0:?}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<MidiMsg>),

    #[error("Dummy error XXX")]
    Dummy, // @XXX
}
