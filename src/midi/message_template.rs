use midly::{live::LiveEvent, MidiMessage};
use serde_with::{serde_as, OneOrMany};
use try_match::match_ok;

use crate::message_template::{self, Matches};

#[serde_as]
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageTemplate {
    NoteOn {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<message_template::Number>,
    },
    NoteOff {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<message_template::Number>,
    },
    ControlChange {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        controller: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        value: Vec<message_template::Number>,
    },
    ProgramChange {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        program: Vec<message_template::Number>,
    },
    PolyPressure {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<message_template::Number>,
    },
    ChannelPressure {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<message_template::Number>,
    },
    PitchBend {
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<message_template::Number>,
        #[serde(default = "message_template::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        bend: Vec<message_template::Number>,
    },
}

impl Matches for MessageTemplate {
    type Message = LiveEvent<'static>;
    type Match = Match;

    /// Checks if the given message matches the template,
    /// and if it does,
    /// returns a [`Match`] describing the qualities of the match.
    fn matches(&self, live_event: LiveEvent<'static>) -> Option<Match> {
        let (channel, message) = match_ok!(
            live_event,
            LiveEvent::Midi {
                channel: _0,
                message: _1
            }
        )?;

        match message {
            MidiMessage::NoteOn { key, vel } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::NoteOn {
                        channel,
                        note,
                        velocity
                    }
                )?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let note = message_template::matches_many(template.note, key.as_int() as u32)?;
                let velocity =
                    message_template::matches_many(template.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOn {
                    channel,
                    note,
                    velocity,
                })
            }

            MidiMessage::NoteOff { key, vel } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::NoteOff {
                        channel,
                        note,
                        velocity
                    }
                )?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let note = message_template::matches_many(template.note, key.as_int() as u32)?;
                let velocity =
                    message_template::matches_many(template.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOff {
                    channel,
                    note,
                    velocity,
                })
            }

            MidiMessage::Aftertouch { key, vel } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::PolyPressure {
                        channel,
                        note,
                        pressure
                    }
                )?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let note = message_template::matches_many(template.note, key.as_int() as u32)?;
                let pressure =
                    message_template::matches_many(template.pressure, vel.as_int() as u32)?;
                Some(Match::PolyPressure {
                    channel,
                    note,
                    pressure,
                })
            }

            MidiMessage::Controller { controller, value } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::ControlChange {
                        channel,
                        controller,
                        value
                    }
                )?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let controller = message_template::matches_many(
                    template.controller,
                    controller.as_int() as u32,
                )?;
                let value = message_template::matches_many(template.value, value.as_int() as u32)?;
                Some(Match::ControlChange {
                    channel,
                    controller,
                    value,
                })
            }

            MidiMessage::ProgramChange { program } => {
                let template =
                    match_ok!(self, MessageTemplate::ProgramChange { channel, program })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let program =
                    message_template::matches_many(template.program, program.as_int() as u32)?;
                Some(Match::ProgramChange { channel, program })
            }

            MidiMessage::ChannelAftertouch { vel } => {
                let template =
                    match_ok!(self, MessageTemplate::ChannelPressure { channel, pressure })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let pressure =
                    message_template::matches_many(template.pressure, vel.as_int() as u32)?;
                Some(Match::ChannelPressure { channel, pressure })
            }

            MidiMessage::PitchBend { bend } => {
                let template = match_ok!(self, MessageTemplate::PitchBend { channel, bend })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let bend = message_template::matches_many(template.bend, bend.0.as_int() as u32)?;
                Some(Match::PitchBend { channel, bend })
            }
        }
    }

    /// Given the qualities of a matched message,
    /// generates the appropriate output message.
    fn generate(&self, matched: Match) -> Option<LiveEvent<'static>> {
        match matched {
            Match::NoteOn {
                channel: (channel_ix, channel_match),
                note: (note_ix, note_match),
                velocity: (velocity_ix, velocity_match),
            } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::NoteOn {
                        channel,
                        note,
                        velocity
                    }
                )?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = template.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = template
                    .velocity
                    .get(velocity_ix as usize)?
                    .generate(velocity_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::NoteOn {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::NoteOff {
                channel: (channel_ix, channel_match),
                note: (note_ix, note_match),
                velocity: (velocity_ix, velocity_match),
            } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::NoteOff {
                        channel,
                        note,
                        velocity
                    }
                )?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = template.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = template
                    .velocity
                    .get(velocity_ix as usize)?
                    .generate(velocity_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::NoteOff {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::ControlChange {
                channel: (channel_ix, channel_match),
                controller: (controller_ix, controller_match),
                value: (value_ix, value_match),
            } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::ControlChange {
                        channel,
                        controller,
                        value
                    }
                )?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let controller = template
                    .controller
                    .get(controller_ix as usize)?
                    .generate(controller_match)?;
                let value = template
                    .value
                    .get(value_ix as usize)?
                    .generate(value_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::Controller {
                        controller: (controller as u8).into(),
                        value: (value as u8).into(),
                    },
                })
            }

            Match::PolyPressure {
                channel: (channel_ix, channel_match),
                note: (note_ix, note_match),
                pressure: (pressure_ix, pressure_match),
            } => {
                let template = match_ok!(
                    self,
                    MessageTemplate::PolyPressure {
                        channel,
                        note,
                        pressure
                    }
                )?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = template.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = template
                    .pressure
                    .get(pressure_ix as usize)?
                    .generate(pressure_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::Aftertouch {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::ProgramChange {
                channel: (channel_ix, channel_match),
                program: (program_ix, program_match),
            } => {
                let template =
                    match_ok!(self, MessageTemplate::ProgramChange { channel, program })?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let program = template
                    .program
                    .get(program_ix as usize)?
                    .generate(program_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::ProgramChange {
                        program: (program as u8).into(),
                    },
                })
            }

            Match::ChannelPressure {
                channel: (channel_ix, channel_match),
                pressure: (pressure_ix, pressure_match),
            } => {
                let template =
                    match_ok!(self, MessageTemplate::ChannelPressure { channel, pressure })?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let vel = template
                    .pressure
                    .get(pressure_ix as usize)?
                    .generate(pressure_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::ChannelAftertouch {
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::PitchBend {
                channel: (channel_ix, channel_match),
                bend: (bend_ix, bend_match),
            } => {
                let template = match_ok!(self, MessageTemplate::PitchBend { channel, bend })?;

                let channel = template
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let bend = template.bend.get(bend_ix as usize)?.generate(bend_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::PitchBend {
                        bend: midly::PitchBend((bend as u16).into()),
                    },
                })
            }
        }
    }
}

/// Contains the information returned when a MIDI message is matched against a [`MessageTemplate`].
/// Each field contains a tuple of
/// the index of the matched value in the `MessageTemplate`'s `Vec`,
/// and the [`NumberMatch`](message_template::NumberMatch) containing info about the matched value.
#[derive(Debug, Clone)]
pub enum Match {
    NoteOn {
        channel: (u32, message_template::NumberMatch),
        note: (u32, message_template::NumberMatch),
        velocity: (u32, message_template::NumberMatch),
    },
    NoteOff {
        channel: (u32, message_template::NumberMatch),
        note: (u32, message_template::NumberMatch),
        velocity: (u32, message_template::NumberMatch),
    },
    ControlChange {
        channel: (u32, message_template::NumberMatch),
        controller: (u32, message_template::NumberMatch),
        value: (u32, message_template::NumberMatch),
    },
    ProgramChange {
        channel: (u32, message_template::NumberMatch),
        program: (u32, message_template::NumberMatch),
    },
    PolyPressure {
        channel: (u32, message_template::NumberMatch),
        note: (u32, message_template::NumberMatch),
        pressure: (u32, message_template::NumberMatch),
    },
    ChannelPressure {
        channel: (u32, message_template::NumberMatch),
        pressure: (u32, message_template::NumberMatch),
    },
    PitchBend {
        channel: (u32, message_template::NumberMatch),
        bend: (u32, message_template::NumberMatch),
    },
}
