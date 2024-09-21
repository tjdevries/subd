use crate::duration;

use crate::move_source::Sign;
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

pub fn sign_serializer<S>(sign: &Sign, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match sign {
        Sign::Nothing => s.serialize_str(" "),
        Sign::Positive => s.serialize_str("+"),
        Sign::Negative => s.serialize_str("-"),
        Sign::Multiply => s.serialize_str("*"),
        Sign::Divide => s.serialize_str("/"),
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    #[serde(serialize_with = "sign_serializer")]
    pub y_sign: Sign,
    #[serde(serialize_with = "sign_serializer")]
    pub x_sign: Sign,
}

impl Coordinates {
    pub fn new(x: Option<f32>, y: Option<f32>) -> Self {
        Self {
            x: x.unwrap_or(0.0),
            y: y.unwrap_or(0.0),
            x_sign: Sign::Nothing,
            y_sign: Sign::Nothing,
        }
    }

    pub fn with_signs(&self, sign: Sign) -> Self {
        Self {
            x: self.x,
            y: self.y,
            x_sign: sign,
            y_sign: sign,
        }
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
