use crate::duration::EasingDuration;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSourceSetting {
    pub source: String,

    pub setting_float: f32,

    pub setting_name: String,

    #[serde(serialize_with = "single_setting")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

impl SingleSourceSetting {
    pub fn new(
        source: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float: f32,
    ) -> Self {
        Self {
            source: source.into(),
            setting_name: setting_name.into(),
            setting_float,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    #[serde(rename = "filter")]
    pub target_filter: String,

    pub setting_float: f32,

    pub setting_name: String,

    #[serde(serialize_with = "single_setting")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

impl SingleSetting {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float,
            duration,
            ..Default::default()
        }
    }
}

fn single_setting<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::SingleSetting as i32)
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    #[serde(rename = "filter")]
    pub target_filter: String,

    #[serde(serialize_with = "settings")]
    pub move_value_type: (),

    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

pub struct SettingsBuilder<T> {
    pub target_filter: String,
    pub settings: Option<T>,
    pub duration: EasingDuration,
}

impl<T: serde::Serialize + std::default::Default> Settings<T> {
    pub fn new(
        target_filter: impl Into<String>,
        settings: T,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            settings,
            duration,
            ..Default::default()
        }
    }
}

impl Add {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float,
            duration,
            ..Default::default()
        }
    }
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Add {
    #[serde(rename = "filter")]
    pub target_filter: String,

    // This could always be the value
    #[serde(serialize_with = "add")]
    move_value_type: (),

    // Might need to be optional
    pub setting_float: f32,

    pub setting_name: String,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Random {
    #[serde(rename = "filter")]
    pub target_filter: String,

    #[serde(serialize_with = "random")]
    move_value_type: (),

    pub setting_name: String,
    pub setting_float_min: f32,
    pub setting_float_max: f32,

    // Takes a min and max value
    #[serde(flatten)]
    pub duration: EasingDuration,
}

impl Random {
    pub fn new(
        target_filter: impl Into<String>,
        setting_name: impl Into<String>,
        setting_float_min: f32,
        setting_float_max: f32,
        duration: EasingDuration,
    ) -> Self {
        Self {
            target_filter: target_filter.into(),
            setting_name: setting_name.into(),
            setting_float_min,
            setting_float_max,
            duration,
            ..Default::default()
        }
    }
}

// Has to be on a typing source
// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Typing {
    pub filter: String,

    // This could always be the value
    #[serde(serialize_with = "typing")]
    move_value_type: (),

    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
enum MoveValueType {
    SingleSetting = 0,
    Settings = 1,
    Random = 2,
    Add = 3,
    Typing = 4,
}

// I feel like we don't need this
fn settings<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Settings as i32)
}

fn random<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Random as i32)
}

fn add<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Add as i32)
}

fn typing<S: Serializer>(_: &(), s: S) -> Result<S::Ok, S::Error> {
    s.serialize_i32(MoveValueType::Typing as i32)
}
