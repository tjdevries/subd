use crate::three_d_filter::CameraMode;
use crate::three_d_filter::FilterName;
use serde::{Deserialize, Serialize};

impl FilterName for ThreeDTransformCornerPin {
    // This should come from some constant
    fn filter_name(&self) -> String {
        "3D-Transform-CornerPin".to_string()
    }
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
