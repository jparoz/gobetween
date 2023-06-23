use midly::{live::LiveEvent, MidiMessage};
use serde_with::{serde_as, OneOrMany};
use try_match::match_ok;

use crate::spec::{self, Matches};

#[serde_as]
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Spec {
    NoteOn {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<spec::Number>,
    },
    NoteOff {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<spec::Number>,
    },
    ControlChange {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        controller: Vec<spec::Number>,
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        value: Vec<spec::Number>,
    },
    ProgramChange {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        program: Vec<spec::Number>,
    },
    PolyPressure {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<spec::Number>,
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<spec::Number>,
    },
    ChannelPressure {
        #[serde(default)]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<spec::Number>,
    },
    PitchBend {
        #[serde(default)]
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
        let msg = match live_event {
            // @Todo: use channel
            LiveEvent::Midi { channel, message } => message,
            _ => return None,
        };

        match msg {
            MidiMessage::NoteOn { key, vel } => {
                let spec = match_ok!(self, Spec::NoteOn { note, velocity })?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let velocity = spec::matches_many(spec.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOn { note, velocity })
            }

            MidiMessage::NoteOff { key, vel } => {
                let spec = match_ok!(self, Spec::NoteOff { note, velocity })?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let velocity = spec::matches_many(spec.velocity, vel.as_int() as u32)?;
                Some(Match::NoteOff { note, velocity })
            }

            MidiMessage::Aftertouch { key, vel } => {
                let spec = match_ok!(self, Spec::PolyPressure { note, pressure })?;
                let note = spec::matches_many(spec.note, key.as_int() as u32)?;
                let pressure = spec::matches_many(spec.pressure, vel.as_int() as u32)?;
                Some(Match::PolyPressure { note, pressure })
            }

            MidiMessage::Controller { controller, value } => {
                let spec = match_ok!(self, Spec::ControlChange { controller, value })?;
                let controller = spec::matches_many(spec.controller, controller.as_int() as u32)?;
                let value = spec::matches_many(spec.value, value.as_int() as u32)?;
                Some(Match::ControlChange { controller, value })
            }

            MidiMessage::ProgramChange { program } => {
                let spec = match_ok!(self, Spec::ProgramChange { program })?;
                let program = spec::matches_many(spec, program.as_int() as u32)?;
                Some(Match::ProgramChange { program })
            }

            MidiMessage::ChannelAftertouch { vel } => {
                let spec = match_ok!(self, Spec::ChannelPressure { pressure })?;
                let pressure = spec::matches_many(spec, vel.as_int() as u32)?;
                Some(Match::ChannelPressure { pressure })
            }

            MidiMessage::PitchBend { bend } => {
                let spec = match_ok!(self, Spec::PitchBend { bend })?;
                let bend = spec::matches_many(spec, bend.0.as_int() as u32)?;
                Some(Match::PitchBend { bend })
            }
        }
    }

    /// Given the qualities of a matched message,
    /// generates the appropriate output message.
    fn generate(&self, matched: Match) -> Option<LiveEvent<'static>> {
        match matched {
            Match::NoteOn {
                note: (note_ix, note_match),
                velocity: (velocity_ix, velocity_match),
            } => {
                let spec = match_ok!(self, Spec::NoteOn { note, velocity })?;

                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
                    .velocity
                    .get(velocity_ix as usize)?
                    .generate(velocity_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::NoteOff {
                note: (note_ix, note_match),
                velocity: (velocity_ix, velocity_match),
            } => {
                let spec = match_ok!(self, Spec::NoteOff { note, velocity })?;

                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
                    .velocity
                    .get(velocity_ix as usize)?
                    .generate(velocity_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::NoteOff {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::ControlChange {
                controller: (controller_ix, controller_match),
                value: (value_ix, value_match),
            } => {
                let spec = match_ok!(self, Spec::ControlChange { controller, value })?;

                let controller = spec
                    .controller
                    .get(controller_ix as usize)?
                    .generate(controller_match)?;
                let value = spec.value.get(value_ix as usize)?.generate(value_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::Controller {
                        controller: (controller as u8).into(),
                        value: (value as u8).into(),
                    },
                })
            }

            Match::PolyPressure {
                note: (note_ix, note_match),
                pressure: (pressure_ix, pressure_match),
            } => {
                let spec = match_ok!(self, Spec::PolyPressure { note, pressure })?;

                let key = spec.note.get(note_ix as usize)?.generate(note_match)?;
                let vel = spec
                    .pressure
                    .get(pressure_ix as usize)?
                    .generate(pressure_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::Aftertouch {
                        key: (key as u8).into(),
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::ProgramChange {
                program: (program_ix, program_match),
            } => {
                let spec = match_ok!(self, Spec::ProgramChange { program })?;

                let program = spec.get(program_ix as usize)?.generate(program_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::ProgramChange {
                        program: (program as u8).into(),
                    },
                })
            }

            Match::ChannelPressure {
                pressure: (pressure_ix, pressure_match),
            } => {
                let spec = match_ok!(self, Spec::ProgramChange { program })?;

                let vel = spec.get(pressure_ix as usize)?.generate(pressure_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
                    message: MidiMessage::ChannelAftertouch {
                        vel: (vel as u8).into(),
                    },
                })
            }

            Match::PitchBend {
                bend: (bend_ix, bend_match),
            } => {
                let spec = match_ok!(self, Spec::PitchBend { bend })?;

                let bend = spec.get(bend_ix as usize)?.generate(bend_match)?;

                Some(LiveEvent::Midi {
                    // @XXX @Fixme: do something with channel
                    channel: 0.into(),
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
        note: (u32, spec::NumberMatch),
        velocity: (u32, spec::NumberMatch),
    },
    NoteOff {
        note: (u32, spec::NumberMatch),
        velocity: (u32, spec::NumberMatch),
    },
    ControlChange {
        controller: (u32, spec::NumberMatch),
        value: (u32, spec::NumberMatch),
    },
    ProgramChange {
        program: (u32, spec::NumberMatch),
    },
    PolyPressure {
        note: (u32, spec::NumberMatch),
        pressure: (u32, spec::NumberMatch),
    },
    ChannelPressure {
        pressure: (u32, spec::NumberMatch),
    },
    PitchBend {
        bend: (u32, spec::NumberMatch),
    },
}
