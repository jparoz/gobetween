use crate::midi;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Spec {
    Midi(midi::Spec),
}

/// Represents a specification for a number or range of numbers,
/// e.g. the velocity value of a MIDI note on message.
#[derive(serde::Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum Number {
    #[default]
    Any,
    Value(u32),
    Range(String),
}
