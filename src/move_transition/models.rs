use serde::{Deserialize, Serialize};

// This is used inside of OBS Messages
// It also does more than Move
// This is related to chat
#[derive(Default, Debug)]
pub struct ChatMoveSourceRequest {
    pub source: String,
    pub scene: String,
    pub x: f32,
    pub y: f32,
    pub rotation_z: f32,
    pub duration: u64,
    pub easing_type: String,
    pub easing_function: String,
    pub easing_type_index: i32,
    pub easing_function_index: i32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

// TODO: Finish this
// Then we need to add defaults
pub struct MoveTimingSettings {
    pub duration: Option<i32>,
    pub easing_function_index: Option<i32>,
    pub easing_type_index: Option<i32>,
}
