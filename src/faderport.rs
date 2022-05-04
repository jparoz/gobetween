use midir::{ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub struct FaderPort {
    midi_name: String,
    midi_input: MidiInputConnection<()>,
    midi_output: MidiOutputConnection,
}

impl FaderPort {
    pub fn new(midi_name: &str) -> Result<Self, Error> {
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
            midi_input: input_connection.ok_or(Error::MidiInputPortNotFound)?,
            midi_output: output_connection.ok_or(Error::MidiOutputPortNotFound)?,
        })
    }
}

pub enum Error {
    MidiInputConnectError(midir::ConnectError<MidiInput>),
    MidiOutputConnectError(midir::ConnectError<MidiOutput>),
    MidiInputPortNotFound,
    MidiOutputPortNotFound,
    MidiInitError(midir::InitError),
    MidiPortInfoError(midir::PortInfoError),
}

impl From<midir::ConnectError<MidiInput>> for Error {
    fn from(e: midir::ConnectError<MidiInput>) -> Error {
        Error::MidiInputConnectError(e)
    }
}

impl From<midir::ConnectError<MidiOutput>> for Error {
    fn from(e: midir::ConnectError<MidiOutput>) -> Error {
        Error::MidiOutputConnectError(e)
    }
}

impl From<midir::InitError> for Error {
    fn from(e: midir::InitError) -> Error {
        Error::MidiInitError(e)
    }
}

impl From<midir::PortInfoError> for Error {
    fn from(e: midir::PortInfoError) -> Error {
        Error::MidiPortInfoError(e)
    }
}
