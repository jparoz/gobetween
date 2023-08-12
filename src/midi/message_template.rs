use midly::{live::LiveEvent, MidiMessage};
use serde_with::{serde_as, OneOrMany};
use try_match::match_ok;

use crate::{
    config::Number,
    message_template::{self, Template},
    transformer::Match,
};

#[serde_as]
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessageTemplate {
    NoteOn {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<Number>,
    },
    NoteOff {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        velocity: Vec<Number>,
    },
    ControlChange {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        controller: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        value: Vec<Number>,
    },
    ProgramChange {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        program: Vec<Number>,
    },
    PolyPressure {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        note: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<Number>,
    },
    ChannelPressure {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        pressure: Vec<Number>,
    },
    PitchBend {
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        channel: Vec<Number>,
        #[serde(default = "Number::default_vec")]
        #[serde_as(as = "OneOrMany<_>")]
        bend: Vec<Number>,
    },
}

impl Template for MessageTemplate {
    type Message = LiveEvent<'static>;

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

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("note".to_string(), note),
                    ("velocity".to_string(), velocity),
                ]))
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

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("note".to_string(), note),
                    ("velocity".to_string(), velocity),
                ]))
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

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("note".to_string(), note),
                    ("pressure".to_string(), pressure),
                ]))
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

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("controller".to_string(), controller),
                    ("value".to_string(), value),
                ]))
            }

            MidiMessage::ProgramChange { program } => {
                let template =
                    match_ok!(self, MessageTemplate::ProgramChange { channel, program })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let program =
                    message_template::matches_many(template.program, program.as_int() as u32)?;

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("program".to_string(), program),
                ]))
            }

            MidiMessage::ChannelAftertouch { vel } => {
                let template =
                    match_ok!(self, MessageTemplate::ChannelPressure { channel, pressure })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let pressure =
                    message_template::matches_many(template.pressure, vel.as_int() as u32)?;

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("pressure".to_string(), pressure),
                ]))
            }

            MidiMessage::PitchBend { bend } => {
                let template = match_ok!(self, MessageTemplate::PitchBend { channel, bend })?;
                let channel =
                    message_template::matches_many(template.channel, channel.as_int() as u32)?;
                let bend = message_template::matches_many(template.bend, bend.0.as_int() as u32)?;

                Some(Match::from_iter([
                    ("channel".to_string(), channel),
                    ("bend".to_string(), bend),
                ]))
            }
        }
    }

    /// Given the qualities of a matched message,
    /// generates the appropriate output message.
    fn generate(&self, mut matched: Match) -> Option<LiveEvent<'static>> {
        match self {
            MessageTemplate::NoteOn {
                channel,
                note,
                velocity,
            } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (note_ix, note_match) = matched.remove("note")?;
                let (velocity_ix, velocity_match) = matched.remove("velocity")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let key = note.get(note_ix as usize)?.generate(note_match)?;
                let vel = velocity
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

            MessageTemplate::NoteOff {
                channel,
                note,
                velocity,
            } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (note_ix, note_match) = matched.remove("note")?;
                let (velocity_ix, velocity_match) = matched.remove("velocity")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let key = note.get(note_ix as usize)?.generate(note_match)?;
                let vel = velocity
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

            MessageTemplate::ControlChange {
                channel,
                controller,
                value,
            } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (controller_ix, controller_match) = matched.remove("controller")?;
                let (value_ix, value_match) = matched.remove("value")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let controller = controller
                    .get(controller_ix as usize)?
                    .generate(controller_match)?;
                let value = value.get(value_ix as usize)?.generate(value_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::Controller {
                        controller: (controller as u8).into(),
                        value: (value as u8).into(),
                    },
                })
            }

            MessageTemplate::PolyPressure {
                channel,
                note,
                pressure,
            } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (note_ix, note_match) = matched.remove("note")?;
                let (pressure_ix, pressure_match) = matched.remove("pressure")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let key = note.get(note_ix as usize)?.generate(note_match)?;
                let vel = pressure
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
            MessageTemplate::ProgramChange { channel, program } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (program_ix, program_match) = matched.remove("program")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let program = program.get(program_ix as usize)?.generate(program_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::ProgramChange {
                        program: (program as u8).into(),
                    },
                })
            }
            MessageTemplate::ChannelPressure { channel, pressure } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (pressure_ix, pressure_match) = matched.remove("pressure")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let vel = pressure
                    .get(pressure_ix as usize)?
                    .generate(pressure_match)?;

                Some(LiveEvent::Midi {
                    channel: (channel as u8).into(),
                    message: MidiMessage::ChannelAftertouch {
                        vel: (vel as u8).into(),
                    },
                })
            }
            MessageTemplate::PitchBend { channel, bend } => {
                let (channel_ix, channel_match) = matched.remove("channel")?;
                let (bend_ix, bend_match) = matched.remove("bend")?;

                let channel = channel.get(channel_ix as usize)?.generate(channel_match)?;
                let bend = bend.get(bend_ix as usize)?.generate(bend_match)?;

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
