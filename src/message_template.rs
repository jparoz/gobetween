use crate::{
    config::{Number, Range},
    transformer::{Match, NumberMatch},
};

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
