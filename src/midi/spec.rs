use midly::{live::LiveEvent, MidiMessage};
use serde_with::{serde_as, OneOrMany};
use try_match::match_ok;

use crate::spec::{self, Matches};

#[serde_as]
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Spec {
    NoteOn {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<spec::Number>,
    },
    NoteOff {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<spec::Number>,
    },
    ControlChange {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        controller: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        value: Vec<spec::Number>,
    },
    ProgramChange {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        program: Vec<spec::Number>,
    },
    PolyPressure {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<spec::Number>,
    },
    ChannelPressure {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<spec::Number>,
    },
    PitchBend {
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<spec::Number>,
        #[serde(default = "spec::Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        bend: Vec<spec::Number>,
    },
}

impl Matches for Spec {
    type Message = LiveEvent<'static>;
    type Match = Match;

    /// Checks if the given message matches the spec,
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
                let spec = match_ok!(
                    self,
                    Spec::NoteOn {
                        channel,
                        note,
                        velocity
                    }
                )?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let velocity = spec::matches_many(spec.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOn {
                    channel,
                    note,
                    velocity,
                })
            }

            MidiMessage::NoteOff { key, vel } => {
                let spec = match_ok!(
                    self,
                    Spec::NoteOff {
                        channel,
                        note,
                        velocity
                    }
                )?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let velocity = spec::matches_many(spec.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOff {
                    channel,
                    note,
                    velocity,
                })
            }

            MidiMessage::Aftertouch { key, vel } => {
                let spec = match_ok!(
                    self,
                    Spec::PolyPressure {
                        channel,
                        note,
                        pressure
                    }
                )?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let pressure = spec::matches_many(spec.pressure, vel.as_int() as u32)?;
                Some(Match::PolyPressure {
                    channel,
                    note,
                    pressure,
                })
            }

            MidiMessage::Controller { controller, value } => {
                let spec = match_ok!(
                    self,
                    Spec::ControlChange {
                        channel,
                        controller,
                        value
                    }
                )?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let controller = spec::matches_many(spec.controller, controller.as_int() as u32)?;
                let value = spec::matches_many(spec.value, value.as_int() as u32)?;
                Some(Match::ControlChange {
                    channel,
                    controller,
                    value,
                })
            }

            MidiMessage::ProgramChange { program } => {
                let spec = match_ok!(self, Spec::ProgramChange { channel, program })?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let program = spec::matches_many(spec.program, program.as_int() as u32)?;
                Some(Match::ProgramChange { channel, program })
            }

            MidiMessage::ChannelAftertouch { vel } => {
                let spec = match_ok!(self, Spec::ChannelPressure { channel, pressure })?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let pressure = spec::matches_many(spec.pressure, vel.as_int() as u32)?;
                Some(Match::ChannelPressure { channel, pressure })
            }

            MidiMessage::PitchBend { bend } => {
                let spec = match_ok!(self, Spec::PitchBend { channel, bend })?;
                let channel = spec::matches_many(spec.channel, channel.as_int() as u32)?;
                let bend = spec::matches_many(spec.bend, bend.0.as_int() as u32)?;
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
                let spec = match_ok!(
                    self,
                    Spec::NoteOn {
                        channel,
                        note,
                        velocity
                    }
                )?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
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
                let spec = match_ok!(
                    self,
                    Spec::NoteOff {
                        channel,
                        note,
                        velocity
                    }
                )?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
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
                let spec = match_ok!(
                    self,
                    Spec::ControlChange {
                        channel,
                        controller,
                        value
                    }
                )?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let controller = spec
                    .controller
                    .get(controller_ix as usize)?
                    .generate(controller_match)?;
                let value = spec.value.get(value_ix as usize)?.generate(value_match)?;

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
                let spec = match_ok!(
                    self,
                    Spec::PolyPressure {
                        channel,
                        note,
                        pressure
                    }
                )?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
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
                let spec = match_ok!(self, Spec::ProgramChange { channel, program })?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let program = spec
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
                let spec = match_ok!(self, Spec::ChannelPressure { channel, pressure })?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let vel = spec
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
                let spec = match_ok!(self, Spec::PitchBend { channel, bend })?;

                let channel = spec
                    .channel
                    .get(channel_ix as usize)?
                    .generate(channel_match)?;
                let bend = spec.bend.get(bend_ix as usize)?.generate(bend_match)?;

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

/// Contains the information returned when a MIDI message is matched against a [`Spec`].
/// Each field contains a tuple of
/// the index of the matched value in the `Spec`'s `Vec`,
/// and the [`NumberMatch`](spec::NumberMatch) containing info about the matched value.
#[derive(Debug, Clone)]
pub enum Match {
    NoteOn {
        channel: (u32, spec::NumberMatch),
        note: (u32, spec::NumberMatch),
        velocity: (u32, spec::NumberMatch),
    },
    NoteOff {
        channel: (u32, spec::NumberMatch),
        note: (u32, spec::NumberMatch),
        velocity: (u32, spec::NumberMatch),
    },
    ControlChange {
        channel: (u32, spec::NumberMatch),
        controller: (u32, spec::NumberMatch),
        value: (u32, spec::NumberMatch),
    },
    ProgramChange {
        channel: (u32, spec::NumberMatch),
        program: (u32, spec::NumberMatch),
    },
    PolyPressure {
        channel: (u32, spec::NumberMatch),
        note: (u32, spec::NumberMatch),
        pressure: (u32, spec::NumberMatch),
    },
    ChannelPressure {
        channel: (u32, spec::NumberMatch),
        pressure: (u32, spec::NumberMatch),
    },
    PitchBend {
        channel: (u32, spec::NumberMatch),
        bend: (u32, spec::NumberMatch),
    },
}
