use crate::three_d_filter::CameraMode;
use crate::three_d_filter::FilterName;
use serde::{Deserialize, Serialize};

// This should come from some constant
impl FilterName for ThreeDTransformCornerPin {
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
    camera_mode: CameraMode,
}

#[derive(Debug, Default)]
pub struct ThreeDTransformCornerPinBuilder {
    pub bottom_left_x: Option<f32>,
    pub bottom_left_y: Option<f32>,
    pub top_left_x: Option<f32>,
    pub top_left_y: Option<f32>,
    pub top_right_x: Option<f32>,
    pub top_right_y: Option<f32>,
}

// TODO: Finish
impl ThreeDTransformCornerPinBuilder {
    pub fn build(self) -> ThreeDTransformCornerPin {
        ThreeDTransformCornerPin {
            ..Default::default()
        }
    }
}
