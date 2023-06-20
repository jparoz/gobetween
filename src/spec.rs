use std::str::FromStr;

use serde_with::{serde_as, DisplayFromStr};

use crate::midi;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Spec {
    Midi(midi::Spec),
}

/// Represents a specification for a number or range of numbers,
/// e.g. the velocity value of a MIDI note on message.
#[serde_as]
#[derive(serde::Deserialize, Debug, Clone, Default)]
#[serde(untagged)]
pub enum Number {
    #[default]
    Any,
    Value(u32),
    Range(#[serde_as(as = "DisplayFromStr")] Range),
}

/// An inclusive range between two numbers,
/// parsed from e.g. `3-10`.
#[derive(Debug, Clone, Default)]
pub struct Range(u32, u32);

impl FromStr for Range {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.trim().split('-');
        let a = iter.next().ok_or("No `-`")?.trim();
        let b = iter.next().ok_or("No number after `-`")?.trim();
        if iter.next().is_none() {
            Ok(Self(
                a.parse()
                    .map_err(|_| "Couldn't parse number on left of `-`")?,
                b.parse()
                    .map_err(|_| "Couldn't parse number on right of `-`")?,
            ))
        } else {
            Err("Too many `-`s")
        }
    }
}
