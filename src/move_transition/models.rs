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

// THESE EXTRA VALUES ARE BULLSHIT!!!
// WE NEED TO ABSTRACT THEM AWAY
// TODO: We need to organize this by:
//       - generic values
//       - values per filter-type
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSingleValueSetting {
    #[serde(rename = "source")]
    pub source: Option<String>,

    #[serde(rename = "filter")]
    pub filter: String,
    #[serde(rename = "duration")]
    pub duration: Option<u32>,
    // #[serde(rename = "move_value_type", default=Some(0))]
    pub move_value_type: Option<u32>,

    #[serde(rename = "setting_float")]
    pub setting_float: f32,
    #[serde(rename = "setting_float_max")]
    pub setting_float_max: f32,
    #[serde(rename = "setting_float_min")]
    pub setting_float_min: f32,
    #[serde(rename = "setting_name")]
    pub setting_name: String,
    #[serde(rename = "value_type")]
    pub value_type: u32,

    // Just for the Blur Filter
    #[serde(rename = "Filter.Blur.Size")]
    pub filter_blur_size: Option<f32>,

    // Just for the SDF Effects Filter
    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    pub glow_inner: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    pub glow_outer: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    pub shadow_outer: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    pub shadow_inner: Option<bool>,
    #[serde(rename = "Filter.SDFEffects.Outline")]
    pub outline: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

// This is wrong IMO
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceFilterSettings {
    pub crop: Option<MoveSourceCropSetting>,

    pub bounds: Option<Coordinates>,

    #[serde(rename = "pos")]
    pub position: Option<Coordinates>,

    pub scale: Option<Coordinates>,

    // This should not be on here
    #[serde(rename = "Rotation.Z")]
    // pub rotation_z: Option<f32>,
    pub duration: Option<u64>,

    pub source: Option<String>,

    // This should be a method on this struct
    // How do we calculate the settings to this string
    //     "transform_text": "pos: x 83.0 y 763.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0",
    pub transform_text: Option<String>,

    // "easing_function_match": Number(10), "easing_match": Number(2),
    #[serde(rename = "easing_function_match")]
    pub easing_function: Option<i32>,
    #[serde(rename = "easing_match")]
    pub easing_type: Option<i32>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceCropSetting {
    #[serde(rename = "bottom")]
    pub bottom: Option<f32>,

    #[serde(rename = "left")]
    pub left: Option<f32>,

    #[serde(rename = "top")]
    pub top: Option<f32>,

    #[serde(rename = "right")]
    pub right: Option<f32>,
}
