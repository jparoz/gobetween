use std::str::FromStr;

use serde_with::{serde_as, DisplayFromStr};

use crate::midi;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Spec {
    Midi(midi::Spec),
}

pub trait Matches {
    type Message;
    type Match;

    /// Checks if the given message matches the spec,
    /// and if it does,
    /// returns a [`Match`](Self::Match) describing the qualities of the match.
    fn matches(&self, msg: Self::Message) -> Option<Self::Match>;

    /// Given the qualities of a matched message,
    /// generates the appropriate output message.
    fn generate(&self, matched: Self::Match) -> Option<Self::Message>;
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

/// The information returned when part of a message is matched against a [`Number`].
#[derive(Debug, Clone)]
pub enum NumberMatch {
    /// Contains the matched value.
    Value(u32),

    /// Contains a float between 0 and 1,
    /// denoting the position in the range which was matched;
    /// 0 meaning the beginning of the range,
    /// and 1 meaning the end of the range.
    Range(f64),
}

impl Number {
    pub fn matches(&self, n: u32) -> Option<NumberMatch> {
        match self {
            Number::Any => Some(NumberMatch::Value(n)),
            Number::Value(m) if n == *m => Some(NumberMatch::Value(n)),
            Number::Range(Range(a, b)) if *a <= n && n <= *b => {
                let a = *a as f64;
                let b = *b as f64;
                let n = n as f64;
                Some(NumberMatch::Range((b - a) / (n - a)))
            }
            _ => None,
        }
    }

    pub fn generate(&self, matched: NumberMatch) -> Option<u32> {
        match (self, matched) {
            (Number::Any, NumberMatch::Value(val)) => Some(val),
            (Number::Any, NumberMatch::Range(_)) => None,

            (Number::Value(val), NumberMatch::Value(_)) => Some(*val),
            (Number::Value(_), NumberMatch::Range(_)) => None,

            (Number::Range(Range(a, b)), NumberMatch::Range(position)) => {
                let a = *a as f64;
                let b = *b as f64;
                let res = a + ((b-a) * position);
                Some(res.round() as u32)
            }
            (Number::Range(_), NumberMatch::Value(_)) => None,
        }
    }
}

/// Tries to match all the [`Number`]s in the `Vec`,
/// and returns the index and [`NumberMatch`] of the first match.
pub fn matches_many<'a, I>(iter: I, n: u32) -> Option<(u32, NumberMatch)>
where
    I: IntoIterator<Item = &'a Number>,
{
    for (i, num) in iter.into_iter().enumerate() {
        if let Some(m) = num.matches(n) {
            return Some((i as u32, m));
        }
    }
    None
}
