use std::ops::Deref;

use bytes::{Buf, BytesMut};
use futures::FutureExt;
use midi_msg::{MidiMsg, ParseError};
use midir::{MidiInput, MidiOutput};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

use crate::device::{self, Device};

impl Device {
    /// Connects to a TCP over MIDI device.
    pub fn tcp_midi(
        join_set: &mut JoinSet<Result<String, device::Error>>,
        name: &str,
        addr: String,
    ) -> Result<Self, device::Error> {
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
                tokio::select! {
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
            mappings: Vec::new(), // @XXX
        })
    }

    /// Connects to a native MIDI device.
    pub fn midi(
        join_set: &mut JoinSet<Result<String, device::Error>>,
        name: &str,
        in_name: &str,
        out_name: &str,
    ) -> Result<Self, device::Error> {
        // @TestMe: is this the right capacity?
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128);
        let (tx, mut rx): (mpsc::Sender<MidiMsg>, mpsc::Receiver<MidiMsg>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        let orig_name = name;

        // @XXX: shouldn't clone these
        let name = name.to_string();
        let in_name = in_name.to_string();
        let out_name = out_name.to_string();

        join_set.spawn(
            async move {
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

                let output_port = output_port
                    .ok_or_else(|| Error::CouldntFindMidiOutput(out_name.to_string()))?;

                // @Checkme: does using "name" make sense here?
                let mut output_connection = output.connect(&output_port, &name)?;

                log::info!("Connected to device {name}");

                while let Some(msg) = rx.recv().await {
                    output_connection.send(&msg.to_midi())?;
                }

                Ok("MIDI device task finished".to_string())
            }
            .map(|res: Result<_, self::Error>| res.map_err(device::Error::from)),
        );

        Ok(Device {
            name: orig_name.to_string(),
            broadcast_tx,
            tx,
            mappings: Vec::new(),
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Channel send error, couldn't send MIDI message {0:?}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<MidiMsg>),

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

    #[error("Couldn't find MIDI input with name: {0}")]
    CouldntFindMidiInput(String),

    #[error("Couldn't find MIDI output with name: {0}")]
    CouldntFindMidiOutput(String),
}
