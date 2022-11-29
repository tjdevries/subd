use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFXCornerPin {
    #[serde(rename = "Camera.Mode")]
    pub camera_mode: Option<i32>,
    #[serde(rename = "Commit")]
    pub commit: String,
    #[serde(rename = "Version")]
    pub version: i64,

    #[serde(rename = "Corners.BottomLeft.X")]
    pub bottom_left_x: Option<f32>,

    #[serde(rename = "Corners.BottomLeft.Y")]
    pub bottom_left_y: Option<f32>,

    #[serde(rename = "Corners.BottomRight.X")]
    pub bottom_right_x: Option<f32>,

    #[serde(rename = "Corners.BottomRight.Y")]
    pub bottom_right_y: Option<f32>,

    #[serde(rename = "Corners.TopLeft.X")]
    pub top_left_x: Option<f32>,

    #[serde(rename = "Corners.TopLeft.Y")]
    pub top_left_y: Option<f32>,

    #[serde(rename = "Corners.TopRight.X")]
    pub top_right_x: Option<f32>,

    #[serde(rename = "Corners.TopRight.Y")]
    pub top_right_y: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFXPerspective {
    #[serde(rename = "Camera.Mode")]
    pub camera_mode: Option<i32>,
    #[serde(rename = "Commit")]
    pub commit: String,
    #[serde(rename = "Version")]
    pub version: i64,

    #[serde(rename = "Camera.FieldOfView")]
    pub field_of_view: Option<f32>,

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
    #[serde(rename = "Position.Z")]
    pub position_z: Option<f32>,

    #[serde(rename = "Rotation.X")]
    pub rotation_x: Option<f32>,
    #[serde(rename = "Rotation.Y")]
    pub rotation_y: Option<f32>,
    #[serde(rename = "Rotation.Z")]
    pub rotation_z: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFXOrthographic {
    #[serde(rename = "Camera.Mode")]
    pub camera_mode: Option<i32>,
    #[serde(rename = "Commit")]
    pub commit: String,
    #[serde(rename = "Version")]
    pub version: i64,

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

// This is the old catch all
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
