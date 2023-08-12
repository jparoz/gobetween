use bytes::BytesMut;
use futures::FutureExt;
use midir::{MidiInput, MidiOutput};
use midly::{live::LiveEvent, stream::MidiStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinSet;

use crate::device::{self, Device};

impl Device<LiveEvent<'static>> {
    /// Connects to a TCP over MIDI device.
    pub fn tcp_midi(
        join_set: &mut JoinSet<Result<String, device::Error>>,
        name: &str,
        addr: String,
    ) -> Result<Self, device::Error> {
        let (broadcast_tx, _broadcast_rx) = broadcast::channel(128); // @TestMe: is this the right capacity?
        let (tx, mut rx): (mpsc::Sender<LiveEvent<'_>>, mpsc::Receiver<_>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();
        let cloned_name = name.to_string();

        join_set.spawn(async move {
            let mut socket = TcpStream::connect(&addr).await?;

            log::info!("Connected to device at address {addr}");

            let mut buf = BytesMut::new();
            let mut out_buf = Vec::new();
            let broadcast_tx = cloned_broadcast_tx;
            let mut stream = MidiStream::new();
            let name = cloned_name;

            loop {
                tokio::select! {
                    bytes_read = socket.read_buf(&mut buf) => {
                        let _bytes_read = bytes_read?;
                        stream.feed(&buf, |live_event| {
                            // Ignore the return value;
                            // error case is when there are no receivers,
                            // which we don't care about.
                            let _ = broadcast_tx.send(live_event.to_static());
                        });

                        // @Note: this relies on the guarantee from BytesMut
                        // that the memory is contiguous.
                        // Specifically,
                        // that BytesMut::Deref<[u8]>
                        // returns all of the contents of the buffer.
                        buf.clear();
                    }
                    Some(live_event) = rx.recv() => {
                        log::trace!("Sending a MIDI message to {name}: {live_event:?}");
                        live_event.write(&mut out_buf).unwrap();
                        socket.write_all(&out_buf).await.unwrap();
                        out_buf.clear();
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
            mapped: Vec::new(),
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
        let (tx, mut rx): (mpsc::Sender<LiveEvent<'_>>, mpsc::Receiver<_>) = mpsc::channel(4);
        let cloned_broadcast_tx = broadcast_tx.clone();

        let orig_name = name;

        // @XXX: shouldn't clone these
        let name = name.to_string();
        let in_name = in_name.to_string();
        let out_name = out_name.to_string();

        join_set.spawn(
            async move {
                let broadcast_tx = cloned_broadcast_tx;
                let mut stream = MidiStream::new();

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
                    move |_timestamp, midi_bytes, ()| {
                        stream.feed(midi_bytes, |live_event| {
                            // Ignore the return value;
                            // error case is when there are no receivers,
                            // which we don't care about.
                            let _ = broadcast_tx.send(live_event.to_static());
                        })
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

                let mut buf = Vec::new();
                while let Some(live_event) = rx.recv().await {
                    live_event.write_std::<&mut [u8]>(buf.as_mut())?;
                    output_connection.send(&buf)?;
                    buf.clear();
                }

                Ok("MIDI device task finished".to_string())
            }
            .map(|res: Result<_, self::Error>| res.map_err(device::Error::from)),
        );

        Ok(Device {
            name: orig_name.to_string(),
            broadcast_tx,
            tx,
            mapped: Vec::new(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Channel send error, couldn't send MIDI message {0:?}")]
    Send(#[from] tokio::sync::mpsc::error::SendError<LiveEvent<'static>>),

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
