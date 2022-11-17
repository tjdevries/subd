use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFXSettings {
    #[serde(rename = "Camera.Mode")]
    pub camera_mode: Option<i32>,

    #[serde(rename = "Commit")]
    pub commit: String,

    #[serde(rename = "Position.X")]
    pub position_x: Option<f32>,

    #[serde(rename = "Position.Y")]
    pub position_y: Option<f32>,

    #[serde(rename = "Position.Z")]
    pub position_z: Option<f32>,

    #[serde(rename = "Rotation.X")]
    pub rotation_x: Option<f32>,

    #[serde(rename = "Rotation.Y")]
    pub rotation_y: Option<f32>,

    #[serde(rename = "Rotation.Z")]
    pub rotation_z: Option<f32>,

    #[serde(rename = "Version")]
    pub version: i64,
}
