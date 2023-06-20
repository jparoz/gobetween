use serde_with::{serde_as, OneOrMany};

use crate::spec;

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
