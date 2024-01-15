use crate::move_transition::duration::EasingDuration;
use serde::{Deserialize, Serialize};

// These all have other types associated with them
enum MoveValueType {
    SingleSetting,
    Settings,
    Random,
    Add,
    Typing,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SingleSetting {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),
    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings<T> {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),
    #[serde(flatten)]
    pub settings: T,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Add {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),

    // This is like the single Settings
    // #[serde(flatten)]
    // pub settings: T,
    #[serde(flatten)]
    pub duration: EasingDuration,
}

// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Random {
    pub filter: String,

    // We need to handle these
    // // These should be Enums
    // #[serde(serialize_with = "value_type")]
    // pub value_type: (),
    // #[serde(serialize_with = "move_value_type")]
    // pub move_value_type: (),

    // This is like the single Settings
    // #[serde(flatten)]
    // pub settings: T,
    //
    // Takes a min and max value
    #[serde(flatten)]
    pub duration: EasingDuration,
}

// Has to be on a typing source
// This is like a single settings
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Typing {
    pub filter: String,

    #[serde(flatten)]
    pub duration: EasingDuration,
}

#[cfg(tests)]
mod tests {
    
    
}

