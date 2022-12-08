use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

impl Default for StreamFXCornerPin {
    fn default() -> Self {
        StreamFXCornerPin {
            camera_mode: Some(0),
            commit: "2099sdd9".to_string(),
            version: 1,
            bottom_left_x: None,
            bottom_left_y: None,
            bottom_right_x: None,
            bottom_right_y: None,
            top_left_x: None,
            top_left_y: None,
            top_right_x: None,
            top_right_y: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl Default for StreamFXPerspective {
    fn default() -> Self {
        StreamFXPerspective {
            camera_mode: Some(0),
            commit: "2099sdd9".to_string(),
            version: 1,
            field_of_view: None,
            scale_x: Some(100.),
            scale_y: Some(100.),
            shear_x: Some(100.),
            shear_y: Some(100.),
            position_x: Some(0.),
            position_y: Some(0.),
            position_z: Some(0.),
            rotation_x: Some(0.),
            rotation_y: Some(0.),
            rotation_z: Some(0.),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl Default for StreamFXOrthographic {
    fn default() -> Self {
        StreamFXOrthographic {
            camera_mode: Some(0),
            commit: "2099sdd9".to_string(),
            version: 1,
            scale_x: Some(100.),
            scale_y: Some(100.),
            shear_x: Some(100.),
            shear_y: Some(100.),
            position_x: Some(0.),
            position_y: Some(0.),
            rotation_x: Some(0.),
            rotation_y: Some(0.),
            rotation_z: Some(0.),
        }
    }
}

// This is the old catch all
#[derive(Serialize, Deserialize, Debug)]
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

impl Default for StreamFXSettings {
    fn default() -> Self {
        StreamFXSettings {
            camera_mode: Some(0),
            commit: "2099sdd9".to_string(),
            version: 1,
            position_x: Some(0.),
            position_y: Some(0.),
            position_z: Some(0.),
            rotation_x: Some(0.),
            rotation_y: Some(0.),
            rotation_z: Some(0.),
        }
    }
}
