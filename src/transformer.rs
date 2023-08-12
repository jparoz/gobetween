use std::collections::HashMap;

use crate::message_template::Template;

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
