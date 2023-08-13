use std::{collections::HashMap, str::FromStr};

use serde_with::{serde_as, DisplayFromStr};

pub trait Template {
    type Message;

    /// Checks if the given message matches the template,
    /// and if it does,
    /// returns a [`Match`](Self::Match) describing the qualities of the match.
    fn matches(&self, msg: Self::Message) -> Option<Match>;

    /// Given the qualities of a matched message,
    /// generates the appropriate output message.
    fn generate(&self, matched: Match) -> Option<Self::Message>;
}

/// Takes an input message and transforms it into the desired output message,
/// if the given input message matches the input template.
pub struct Transformer<Fr, To> {
    // @Todo: none of these should be pub,
    // use a From impl or something similar instead
    pub input: Fr,
    pub output: To,
    pub field_map: HashMap<String, String>,
}

impl<Fr, To> Transformer<Fr, To>
where
    Fr: Template,
    To: Template,
{
    pub fn transform(&self, in_msg: Fr::Message) -> Option<To::Message> {
        self.input.matches(in_msg).and_then(|mat| {
            let mapped_mat = mat
                .into_iter()
                .map(|(field, val)| {
                    let mapped_field = self.field_map.get(&field).cloned().unwrap_or(field);
                    (mapped_field, val)
                })
                .collect();
            self.output.generate(mapped_mat)
        })
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

/// An inclusive range between two numbers,
/// parsed from e.g. `3-10`.
#[derive(Debug, Clone, Default)]
pub struct Range(pub u32, pub u32);

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

impl Number {
    pub fn matches(&self, n: u32) -> Option<NumberMatch> {
        match self {
            Number::Any => Some(NumberMatch::Value(n)),
            Number::Value(m) if n == *m => Some(NumberMatch::Value(n)),
            Number::Range(Range(a, b)) if *a <= n && n <= *b => {
                let a = *a as f64;
                let b = *b as f64;
                let n = n as f64;
                Some(NumberMatch::Range((n - a) / (b - a)))
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
                let res = a + ((b - a) * position);
                Some(res.round() as u32)
            }
            (Number::Range(_), NumberMatch::Value(_)) => None,
        }
    }

    /// Returns the desired default when no `Number` is specified.
    pub fn default_vec() -> Vec<Number> {
        vec![Number::Any]
    }
}

// @Todo @Cleanup:
// This whole manual index-keeping business is annoying.
// It would be better to somehow more closely pair the input->output mapping,
// so that we can directly match each pair of `NumberMatch`es,
// to obviate this function in favour of e.g. itertools::zip_longest.
// (see also https://docs.rs/itertools/latest/itertools/enum.EitherOrBoth.html#method.or_default)
//
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

// @Cleanup: put this in the same place as Number
pub type Match = HashMap<String, (u32, NumberMatch)>;

// @Cleanup: put this in the same place as Number
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
