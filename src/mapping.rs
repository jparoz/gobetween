use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::spec::Spec;

/// A mapping that has been realised,
/// and will be applied to incoming messages.
pub struct Mapped<Message> {
    /// This closure is called on each input message;
    /// if the closure produces a `Message`,
    /// then the message is sent to `tx`.
    pub f: Box<dyn Fn(Message) -> Option<Message>>,
    pub tx: mpsc::Sender<Message>,
}

impl<Message> Mapped<Message> {
    pub fn new(
        trigger: Spec,
        target: Spec,
        tx: mpsc::Sender<Message>,
        field_map: FieldMap,
    ) -> Self {
        Mapped {
            f: Box::new(move |msg| {
                // @Todo:
                // - use Spec::matches to find if the message should be mapped;
                // - use the return value of Spec::matches to make a mapped output message.
                let _trigger = &trigger;
                let _target = &target;
                let _field_map = &field_map;
                todo!();
            }),
            tx,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Mapping {
    #[serde(rename = "from")]
    pub spec: Spec,

    #[serde(rename = "to")]
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Target {
    #[serde(rename = "target")]
    pub name: String,

    #[serde(rename = "mapping")]
    #[serde(default)]
    pub field_map: FieldMap,

    #[serde(flatten)]
    pub spec: Spec,
}

// @Todo: is this the right type? Represents a message field name such as note, controller, etc.
pub type FieldMap = HashMap<String, String>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't find device `{0}` (found in mapping)")]
    DeviceNotFound(String),
}
