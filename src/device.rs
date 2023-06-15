use std::io;
use std::ops::Deref;

use bytes::{Buf, BytesMut};
use midi_msg::{Channel, ChannelVoiceMsg, ControlChange, MidiMsg, ParseError};
use midir::{MidiInput, MidiOutput};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

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
        Ok(self.tx.send(msg).await?)
    }

    fn tcp_midi(
        join_set: &mut JoinSet<Result<String, Error>>,
        name: &str,
        addr: String,
    ) -> Result<Self, Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        join_set.spawn(async move {
            let mut socket = TcpStream::connect(&addr).await?;

            log::info!("Connected to device at address {addr}");

            let mut buf = BytesMut::new();
            let broadcast_tx = cloned_broadcast_tx;
            let mut ctx = midi_msg::ReceiverContext::new();

            loop {
                select! {
                    bytes_read = socket.read_buf(&mut buf) => {
                        let _bytes_read = bytes_read?;
                        parse_midi_from_buf(&mut buf, &mut ctx, &broadcast_tx);
                    }
                    Some(msg) = rx.recv() => {
                        socket.write_all(&msg.to_midi()).await?;
                    }
                    else => { break }
                }
            }

            Ok("TCP MIDI device task finished".to_string())
        });

        Ok(Device {
            name: name.to_string(),
            broadcast_tx,
            tx,
        })
    }

    fn midi(
        join_set: &mut JoinSet<Result<String, Error>>,
        name: &str,
        in_name: &str,
        out_name: &str,
    ) -> Result<Self, Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        let orig_name = name;

        // @XXX: shouldn't clone these
        let name = name.to_string();
        let in_name = in_name.to_string();
        let out_name = out_name.to_string();

        join_set.spawn(async move {
            let broadcast_tx = cloned_broadcast_tx;
            let mut ctx = midi_msg::ReceiverContext::new();

            // @Checkme: does using "name" make sense here?
            let input = MidiInput::new(&name)?;
            let output = MidiOutput::new(&name)?;

            let mut input_port = None;
            for port in input.ports().iter() {
                let name = input.port_name(port)?;
                log::trace!("Found input port: {name}");
                if name == in_name {
                    input_port = Some(port.clone());
                    break;
                }
            }

            let input_port =
                input_port.ok_or_else(|| Error::CouldntFindMidiInput(in_name.to_string()))?;

            let _input_connection = input.connect(
                &input_port,
                // @Checkme: does using "name" make sense here?
                &name,
                move |_timestamp, mut midi_bytes, ()| {
                    // // @Todo @XXX: don't ignore this error, we could get stuck
                    // while let Ok((msg, len)) =
                    //     MidiMsg::from_midi_with_context(midi_bytes, &mut ctx)
                    // {
                    //     // Advance the buffer by the length of the parsed MIDI message.
                    //     midi_bytes = &midi_bytes[len..];

                    //     // Ignore the return value;
                    //     // error case is when there are no receivers,
                    //     // which we don't care about.
                    //     let _ = broadcast_tx.send(msg);
                    // }
                    // @Checkme: Delete the above commented code after testing below
                    parse_midi_from_buf(&mut midi_bytes, &mut ctx, &broadcast_tx);
                },
                (),
            )?;

            let mut output_port = None;
            for port in output.ports().iter() {
                let name = output.port_name(port)?;
                log::trace!("Found output port: {name}");
                if name == out_name {
                    output_port = Some(port.clone());
                    break;
                }
            }

            let output_port =
                output_port.ok_or_else(|| Error::CouldntFindMidiOutput(out_name.to_string()))?;

            // @Checkme: does using "name" make sense here?
            let mut output_connection = output.connect(&output_port, &name)?;

            log::info!("Connected to device {name}");

            while let Some(msg) = rx.recv().await {
                output_connection.send(&msg.to_midi())?;
            }

            Ok("MIDI device task finished".to_string())
        });

        Ok(Device {
            name: orig_name.to_string(),
            broadcast_tx,
            tx,
        })
    }
}

/// Parses as many MIDI messages as possible from the given [`BytesMut`],
/// advancing the buffer as needed,
/// and logging any errors.
fn parse_midi_from_buf<B>(
    buf: &mut B,
    ctx: &mut midi_msg::ReceiverContext,
    tx: &broadcast::Sender<MidiMsg>,
) where
    B: Buf + Deref<Target = [u8]>,
{
    while buf.has_remaining() {
        match MidiMsg::from_midi_with_context(buf, ctx) {
            Ok((msg, len)) => {
                // Advance the buffer by the length of the parsed MIDI message.
                buf.advance(len);

                // Ignore the return value;
                // error case is when there are no receivers,
                // which we don't care about.
                let _ = tx.send(msg);
            }

            // This is okay; just means we haven't received all the bytes yet.
            Err(ParseError::UnexpectedEnd) | Err(ParseError::NoEndOfSystemExclusiveFlag) => break,

            Err(parse_error) => {
                // @Todo: log more information about the source of the error,
                // e.g. which Device originated the error.
                log::error!("MIDI parse error: {parse_error}");

                log::info!("Skipping byte `0x{:02X}` because of previous error", buf[0]);
                buf.advance(1);
            }
        }
    }
}

// @Todo: proper errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] io::Error),

    #[error("MIDI init error: {0}")]
    MidirInit(#[from] midir::InitError),

    #[error("MIDI port info error: {0}")]
    MidirPortInfo(#[from] midir::PortInfoError),

    #[error("MIDI input connection error: {0}")]
    MidirInputConnect(#[from] midir::ConnectError<midir::MidiInput>),

    #[error("MIDI output connection error: {0}")]
    MidirOutputConnect(#[from] midir::ConnectError<midir::MidiOutput>),

    #[error("MIDI send error: {0}")]
    MidirSend(#[from] midir::SendError),

    #[error("Channel send error, couldn't send MIDI message {0:?}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<MidiMsg>),

    #[error("Couldn't find MIDI input with name: {0}")]
    CouldntFindMidiInput(String),

    #[error("Couldn't find MIDI output with name: {0}")]
    CouldntFindMidiOutput(String),
}
