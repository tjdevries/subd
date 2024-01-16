use crate::move_transition::duration;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

// I want to remove this
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MovePluginSettings<T> {
    pub filter: String,

    // I don't know if we need thisT
    // These should be Enums
    #[serde(serialize_with = "value_type")]
    pub value_type: (),

    #[serde(serialize_with = "move_value_type")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: duration::EasingDuration,
}

// This is wrong now
fn value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(2)
}

fn move_value_type<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(1)
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl Coordinates {
    pub fn new(x: Option<f32>, y: Option<f32>) -> Self {
        Self { x, y }
    }
}

#[allow(dead_code)]
enum FilterKind {
    MoveAction,
    MoveValue,
    MoveSource,
}

impl fmt::Display for FilterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FilterKind::MoveAction => write!(f, "move_action_filter"),
            FilterKind::MoveValue => write!(f, "move_value_filter"),
            FilterKind::MoveSource => write!(f, "move_source_filter"),
        }
    }
}
