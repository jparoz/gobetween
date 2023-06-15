use std::io;

use bytes::{Buf, BytesMut};
use midir::{MidiInput, MidiOutput};
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
    /// MIDI over TCP connection information.
    TcpMidi {
        /// The address and port of the TCP MIDI device.
        /// This will be used by [`ToSocketAddrs`],
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
    pub fn connect(&self) -> Result<Device, Error> {
        use ConnectionInfo::*;
        match &self.connection_info {
            TcpMidi { midi_address } => Device::tcp_midi(&self.name, midi_address.to_string()),
            Midi { midi_in, midi_out } => Device::midi(&self.name, midi_in, midi_out),
        }
    }
}

#[derive(Debug)]
pub struct Device {
    /// The name of the device. Can be anything.
    // @Todo: This could probably be a reference into the originating DeviceInfo
    pub name: String,

    /// This is a clone of the sender moved to the main device callback.
    /// Use [`Device::subscribe`] to receive MIDI messages from this device.
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

    fn tcp_midi(name: &str, addr: String) -> Result<Self, Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        tokio::spawn(async move {
            // @Fixme: shouldn't unwrap
            let mut socket = TcpStream::connect(&addr).await.unwrap();

            log::info!("Connected to device at address {addr}");

            let mut buf = BytesMut::new();
            let broadcast_tx = cloned_broadcast_tx;
            let mut ctx = midi_msg::ReceiverContext::new();

            loop {
                select! {
                    bytes_read = socket.read_buf(&mut buf) => {
                        // @Fixme: shouldn't unwrap
                        let _bytes_read = bytes_read.unwrap();

                        // @Todo @XXX: don't ignore this error, we could get stuck
                        while let Ok((msg, len)) =
                            MidiMsg::from_midi_with_context(&buf, &mut ctx)
                        {
                            // Advance the buffer by the length of the parsed MIDI message.
                            buf.advance(len);

                            // Ignore the return value;
                            // error case is when there are no receivers,
                            // which we don't care about.
                            let _ = broadcast_tx.send(msg);
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

    fn midi(name: &str, in_name: &str, out_name: &str) -> Result<Self, Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        let orig_name = name;

        // @XXX: shouldn't clone these
        let name = name.to_string();
        let in_name = in_name.to_string();
        let out_name = out_name.to_string();

        // @Fixme @Todo @XXX: Don't use unwrap in here
        tokio::spawn(async move {
            let broadcast_tx = cloned_broadcast_tx;
            let mut ctx = midi_msg::ReceiverContext::new();

            // @Checkme: does using "name" make sense here?
            let input = MidiInput::new(&name).unwrap();
            let output = MidiOutput::new(&name).unwrap();

            let mut input_port = None;
            for port in input.ports().iter() {
                // @XXX: REALLY DON'T UNWRAP
                let name = input.port_name(port).unwrap();
                log::trace!("Found input port: {name}");
                if name == in_name {
                    input_port = Some(port.clone());
                    break;
                }
            }

            // @XXX: unwrap
            let input_port = input_port
                .ok_or_else(|| Error::CouldntFindMidiInput(in_name.to_string()))
                .unwrap();

            let _input_connection = input
                .connect(
                    &input_port,
                    // @Checkme: does using "name" make sense here?
                    &name,
                    move |_timestamp, mut midi_bytes, ()| {
                        // @Todo @XXX: don't ignore this error, we could get stuck
                        while let Ok((msg, len)) =
                            MidiMsg::from_midi_with_context(midi_bytes, &mut ctx)
                        {
                            // Advance the buffer by the length of the parsed MIDI message.
                            midi_bytes = &midi_bytes[len..];

                            // Ignore the return value;
                            // error case is when there are no receivers,
                            // which we don't care about.
                            let _ = broadcast_tx.send(msg);
                        }
                    },
                    (),
                )
                .unwrap();

            let mut output_port = None;
            for port in output.ports().iter() {
                let name = output.port_name(port).unwrap();
                log::trace!("Found output port: {name}");
                if name == out_name {
                    output_port = Some(port.clone());
                    break;
                }
            }

            let output_port = output_port
                .ok_or_else(|| Error::CouldntFindMidiOutput(out_name.to_string()))
                .unwrap();

            // @Checkme: does using "name" make sense here?
            let mut output_connection = output.connect(&output_port, &name).unwrap();

            log::info!("Connected to device {name}");

            loop {
                let msg = rx.recv().await.unwrap(); // @Fixme: shouldn't unwrap, maybe pattern match?

                // @XXX: unwrap
                output_connection.send(&msg.to_midi()).unwrap();
            }
        });

        Ok(Device {
            name: orig_name.to_string(),
            broadcast_tx,
            tx,
        })
    }
}

// @Todo: proper errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("Channel send error, couldn't send MIDI message {0:?}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<MidiMsg>),

    #[error("Couldn't find MIDI input with name: {0}")]
    CouldntFindMidiInput(String),

    #[error("Couldn't find MIDI output with name: {0}")]
    CouldntFindMidiOutput(String),
}
