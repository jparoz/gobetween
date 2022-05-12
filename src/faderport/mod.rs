use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use thiserror::Error;
use tokio::sync::broadcast;

mod button;
mod encoder;
mod fader;
mod message;

use button::Button;
use encoder::Encoder;
use fader::Fader;
use message::Message;

pub struct FaderPort {
    // MIDI details
    midi_name: String,
    midi_input: MidiInputConnection<()>,
    midi_output: MidiOutputConnection,

    // This is the master receiver which receives FaderPort state update messages
    rx: broadcast::Receiver<Message>,
    // Fader components
    // @Todo
}

impl FaderPort {
    pub fn new(midi_name: &str) -> Result<Self, FaderPortError> {
        let midi_in = MidiInput::new("gobetween_client")?; // @Fixme: Should this be exec name?

        // This is a tuple of (broadcast::Receiver<Message>, MidiInputConnection<()>)
        let mut input_connection = None;

        for port in midi_in.ports().iter() {
            if midi_in.port_name(port)? == midi_name {
                let (tx, rx) = broadcast::channel(128); // @TestMe: is this the right capacity?

                input_connection = Some((
                    rx,
                    midi_in.connect(
                        port,
                        "gobetween_port",
                        move |_timestamp, msg, _data| {
                            println!("Got a MIDI message: {:?}", msg);

                            // Match on the status byte
                            match msg[0] {
                                0x80 => {
                                    // Note on: Button pressed
                                    let button_id = msg[1];
                                    match msg[2] {
                                        0x00 => {
                                            tx.send(Message::ButtonPressed(Button::from_byte(
                                                button_id,
                                            )))
                                            .unwrap();
                                        }
                                        0x7F => {
                                            tx.send(Message::ButtonReleased(Button::from_byte(
                                                button_id,
                                            )))
                                            .unwrap();
                                        }
                                        _ => eprintln!("Invalid button state: 0x{:02X}", msg[2]),
                                    }
                                }

                                0xB0 => {
                                    // Control change: Encoder increment/decrement
                                    let byte = msg[2];
                                    let sign = if byte & 0x40 > 0 { 1i8 } else { -1i8 };
                                    let magnitude = (byte & 0x3F) as i8;
                                    let delta = magnitude * sign;

                                    let encoder_id = msg[1];

                                    tx.send(Message::EncoderRotate(
                                        Encoder::from_byte(encoder_id),
                                        delta,
                                    ))
                                    .unwrap();

                                    println!(
                                        "Encoder ID {} increment/decrement by {}",
                                        encoder_id, delta
                                    );
                                }

                                0xE0..=0xEF => {
                                    // Pitch wheel: Fader level changed
                                    let fader_index = msg[0] & 0x0F;
                                    let fader_value = bit14!(msg[1], msg[2]);

                                    tx.send(Message::FaderLevel(
                                        Fader::from_byte(fader_index),
                                        fader_value,
                                    ))
                                    .unwrap();

                                    println!(
                                        "Fader index {} level changed to {}",
                                        fader_index, fader_value
                                    );
                                }

                                _ => eprintln!("Invalid FaderPort MIDI message: {:?}", msg),
                            }
                        },
                        (),
                    )?,
                ));
                break;
            }
        }

        let midi_out = MidiOutput::new("gobetween_client")?; // @Fixme: Should this be exec name?
        let mut output_connection = None;

        for port in midi_out.ports().iter() {
            if midi_out.port_name(port).unwrap() == midi_name {
                output_connection = Some(midi_out.connect(port, "gobetween_port")?);
                break;
            }
        }

        let (rx, midi_input) = input_connection.ok_or(FaderPortError::MidiInputPortNotFound)?;
        let midi_output = output_connection.ok_or(FaderPortError::MidiOutputPortNotFound)?;

        Ok(FaderPort {
            midi_name: midi_name.to_string(),
            midi_input,
            midi_output,
            rx,
        })
    }
}

#[derive(Error, Debug)]
pub enum FaderPortError {
    // Custom errors produced by us
    #[error("Couldn't find the chosen MIDI input port")]
    MidiInputPortNotFound,

    #[error("Couldn't find the chosen MIDI output port")]
    MidiOutputPortNotFound,

    // Errors from midir
    #[error("MIDI error: {0}")]
    MidiInputConnectError(#[from] midir::ConnectError<MidiInput>),

    #[error("MIDI error: {0}")]
    MidiOutputConnectError(#[from] midir::ConnectError<MidiOutput>),

    #[error("MIDI error: {0}")]
    MidiInitError(#[from] midir::InitError),

    #[error("MIDI error: {0}")]
    MidiPortInfoError(#[from] midir::PortInfoError),
}
