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

// we create Json of What we want
// we then convert to a MoveMultipleStruct
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MoveMultipleValuesSetting {
    pub filter: Option<String>,

    // #[serde(default="multiple_settings_value_type_default")]
    pub move_value_type: u32,

    #[serde(rename = "duration")]
    pub duration: Option<u32>,

    // What is the difference
    // #[serde(default="multiple_settings_value_type_default")]
    pub value_type: u32,

    #[serde(rename = "Camera.FieldOfView")]
    pub field_of_view: Option<f32>,

    // "easing_function_match": Number(10), "easing_match": Number(2),
    #[serde(rename = "easing_function_match")]
    pub easing_function: Option<i32>,
    #[serde(rename = "easing_match")]
    pub easing_type: Option<i32>,

    // This is ortho
    #[serde(rename = "Scale.X")]
    pub scale_x: Option<f32>,
    #[serde(rename = "Scale.Y")]
    pub scale_y: Option<f32>,
    #[serde(rename = "Shear.X")]
    pub shear_x: Option<f32>,
    #[serde(rename = "Shear.Y")]
    pub shear_y: Option<f32>,
    #[serde(rename = "Position.X")]
    pub position_x: Option<f32>,
    #[serde(rename = "Position.Y")]
    pub position_y: Option<f32>,
    #[serde(rename = "Rotation.X")]
    pub rotation_x: Option<f32>,
    #[serde(rename = "Rotation.Y")]
    pub rotation_y: Option<f32>,
    #[serde(rename = "Rotation.Z")]
    pub rotation_z: Option<f32>,
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
