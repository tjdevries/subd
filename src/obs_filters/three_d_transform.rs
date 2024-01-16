use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum SpinFilters {
    Orthographic(ThreeDTransformOrthographic),
    Perspective(ThreeDTransformPerspective),
}

pub trait FilterName {
    fn filter_name(&self) -> String;
}

// This should come from constants
impl FilterName for ThreeDTransformPerspective {
    // This should come from some constant
    fn filter_name(&self) -> String {
        "3D-Transform-Perspective".to_string()
    }
}

impl FilterName for ThreeDTransformOrthographic {
    // This should come from some constant
    fn filter_name(&self) -> String {
        "3D-Transform-Orthographic".to_string()
    }
}

impl FilterName for ThreeDTransformCornerPin {
    // This should come from some constant
    fn filter_name(&self) -> String {
        "3D-Transform-CornerPin".to_string()
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum CameraMode {
    #[default]
    Perspective = 0,
    Orthographic = 1,
    CornerPin = 2,
}

// How should we have a hardcoded value associated with each Struct?
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ThreeDTransformPerspective {
    #[serde(
        rename = "Camera.FieldOfView",
        skip_serializing_if = "Option::is_none"
    )]
    pub field_of_view: Option<f32>,

    #[serde(rename = "Position.X", skip_serializing_if = "Option::is_none")]
    pub position_x: Option<f32>,

    #[serde(rename = "Position.Y", skip_serializing_if = "Option::is_none")]
    pub position_y: Option<f32>,

    #[serde(rename = "Position.Z", skip_serializing_if = "Option::is_none")]
    pub position_z: Option<f32>,

    #[serde(rename = "Rotation.X", skip_serializing_if = "Option::is_none")]
    pub rotation_x: Option<f32>,

    #[serde(rename = "Rotation.Y", skip_serializing_if = "Option::is_none")]
    pub rotation_y: Option<f32>,

    #[serde(rename = "Rotation.Z", skip_serializing_if = "Option::is_none")]
    pub rotation_z: Option<f32>,

    #[serde(rename = "Scale.X", skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,

    #[serde(rename = "Scale.Y", skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,

    #[serde(rename = "Shear.X", skip_serializing_if = "Option::is_none")]
    pub shear_x: Option<f32>,

    #[serde(rename = "Shear.Y", skip_serializing_if = "Option::is_none")]
    pub shear_y: Option<f32>,

    #[serde(rename = "Camera.Mode")]
    pub camera_mode: CameraMode,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ThreeDTransformOrthographic {
    #[serde(rename = "Position.X", skip_serializing_if = "Option::is_none")]
    pub position_x: Option<f32>,

    #[serde(rename = "Position.Y", skip_serializing_if = "Option::is_none")]
    pub position_y: Option<f32>,

    #[serde(rename = "Position.Z", skip_serializing_if = "Option::is_none")]
    pub position_z: Option<f32>,

    #[serde(rename = "Rotation.X", skip_serializing_if = "Option::is_none")]
    pub rotation_x: Option<f32>,

    #[serde(rename = "Rotation.Y", skip_serializing_if = "Option::is_none")]
    pub rotation_y: Option<f32>,

    #[serde(rename = "Rotation.Z", skip_serializing_if = "Option::is_none")]
    pub rotation_z: Option<f32>,

    #[serde(rename = "Scale.X", skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<f32>,

    #[serde(rename = "Scale.Y", skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<f32>,

    #[serde(rename = "Shear.X", skip_serializing_if = "Option::is_none")]
    pub shear_x: Option<f32>,

    #[serde(rename = "Shear.Y", skip_serializing_if = "Option::is_none")]
    pub shear_y: Option<f32>,

    #[serde(rename = "Camera.Mode")]
    pub camera_mode: CameraMode,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ThreeDTransformCornerPin {
    #[serde(
        rename = "Corners.BottomLeft.X",
        skip_serializing_if = "Option::is_none"
    )]
    pub bottom_left_x: Option<f32>,

    #[serde(
        rename = "Corners.BottomRight.Y",
        skip_serializing_if = "Option::is_none"
    )]
    pub bottom_left_y: Option<f32>,

    #[serde(
        rename = "Corners.TopLeft.X",
        skip_serializing_if = "Option::is_none"
    )]
    pub top_left_x: Option<f32>,

    #[serde(
        rename = "Corners.TopLeft.Y",
        skip_serializing_if = "Option::is_none"
    )]
    pub top_left_y: Option<f32>,

    #[serde(
        rename = "Corners.TopRight.X",
        skip_serializing_if = "Option::is_none"
    )]
    pub top_right_x: Option<f32>,

    #[serde(
        rename = "Corners.TopRight.Y",
        skip_serializing_if = "Option::is_none"
    )]
    pub top_right_y: Option<f32>,

    #[serde(rename = "Camera.Mode")]
    pub camera_mode: CameraMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs;
    // use crate::move_transition::models;
    // use crate::move_transition::move_transition;
    // use std::thread;
    // use std::time;

    #[tokio::test]
    async fn test_fun() {
        let obs_client = obs::create_obs_client().await.unwrap();
        let settings = ThreeDTransformPerspective {
            rotation_x: Some(360.0),
            rotation_y: Some(360.0),
            rotation_z: Some(360.0),
            camera_mode: CameraMode::Orthographic,
            ..Default::default()
        };
        println!("{:?}", serde_json::to_string_pretty(&settings));
        // let d = models::DurationSettings {
        //     duration: Some(9000),
        //     ..Default::default()
        // };
        // let _ = move_transition::update_and_trigger_3d_filter(
        //     &obs_client,
        //     "begin",
        //     "3d-Transform-Orthographic",
        //     settings,
        //     d,
        // )
        // .await;
    }
}
