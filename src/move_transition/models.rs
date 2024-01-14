use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

// I think the easing will want to be an Enum
// TODO: Finish this
// Then we need to add defaults
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DurationSettings {
    pub duration: Option<i32>,
    pub easing_function_index: Option<i32>,
    pub easing_type_index: Option<i32>,
    pub easing_type: Option<String>,
    pub easing_function: Option<String>,
}
