use thiserror::Error;

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub struct FaderPort {
    midi_name: String,
    midi_input: MidiInputConnection<()>,
    midi_output: MidiOutputConnection,
}

impl FaderPort {
    pub fn new(midi_name: &str) -> Result<Self, FaderPortError> {
        let midi_in = MidiInput::new("gobetween_client")?; // @Fixme: Should this be exec name?
        let mut input_connection = None;

        for port in midi_in.ports().iter() {
            if midi_in.port_name(port)? == midi_name {
                input_connection = Some(midi_in.connect(
                    port,
                    "gobetween_port",
                    |_timestamp, msg, _data| {
                        println!("Got a MIDI message: {:?}", msg);
                    },
                    (),
                )?);
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

        Ok(FaderPort {
            midi_name: midi_name.to_string(),
            midi_input: input_connection.ok_or(FaderPortError::MidiInputPortNotFound)?,
            midi_output: output_connection.ok_or(FaderPortError::MidiOutputPortNotFound)?,
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
