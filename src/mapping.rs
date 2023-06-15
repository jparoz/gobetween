#[derive(serde::Deserialize, Debug, Clone)]
pub struct Mapping {
    from: Trigger,
    to: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Trigger {
    Midi(MidiTrigger),
}

type MidiTrigger = Midi<MidiTriggerValue>;

#[derive(serde::Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum MidiTriggerValue {
    Value(u16), // @Todo: more precise than u16?

    // Range(std::ops::Range<u16>), // @Todo
    #[default]
    Any,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Target {
    #[serde(rename = "target")]
    name: String,

    #[serde(flatten)]
    target: InnerTarget,
}

// @Todo: better name (or better structure)
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum InnerTarget {
    Midi(MidiTarget),
}

type MidiTarget = Midi<MidiTargetValue>;

#[derive(serde::Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum MidiTargetValue {
    Value(u16), // @Todo: more precise than u16?

    // Range(std::ops::Range<u16>), // @Todo
    Variable(String),

    #[default]
    Any,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Midi<T: Default> {
    NoteOn {
        #[serde(default)]
        note: T,
        #[serde(default)]
        velocity: T,
    },
    NoteOff {
        #[serde(default)]
        note: T,
        #[serde(default)]
        velocity: T,
    },
    ControlChange {
        #[serde(default)]
        controller: T,
        #[serde(default)]
        value: T,
    },
    ProgramChange {
        #[serde(default)]
        program: T,
    },
    PolyPressure {
        #[serde(default)]
        note: T,
        #[serde(default)]
        pressure: T,
    },
    ChannelPressure {
        #[serde(default)]
        pressure: T,
    },
    PitchBend {
        #[serde(default)]
        bend: T,
    },
}
