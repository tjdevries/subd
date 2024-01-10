use serde::{Deserialize, Serialize, Serializer};

enum ThreeDTransformFilters {
    Perspective(ThreeDTransformPerspective),
    Orthographic(ThreeDTransformOrthographic),
    CornerPin(ThreeDTransformCornerPin),
}

// How should we have a hardcoded value associated with each Struct?
#[derive(Serialize, Deserialize, Debug, Default)]
struct ThreeDTransformPerspective {
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

    #[serde(
        serialize_with = "perspective_camera_mode",
        rename = "Camera.Mode"
    )]
    camera_mode: (),
}

fn perspective_camera_mode<S: Serializer>(
    _: &(),
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_i32(1)
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ThreeDTransformOrthographic {
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

    #[serde(
        serialize_with = "orthographic_camera_mode",
        rename = "Camera.Mode"
    )]
    camera_mode: (),
}

fn orthographic_camera_mode<S: Serializer>(
    _: &(),
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_i32(0)
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ThreeDTransformCornerPin {
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

    #[serde(serialize_with = "corner_pin_camera_mode", rename = "Camera.Mode")]
    camera_mode: (),
}

fn corner_pin_camera_mode<S: Serializer>(
    _: &(),
    s: S,
) -> Result<S::Ok, S::Error> {
    s.serialize_i32(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_filters() {
        let settings = ThreeDTransformPerspective {
            field_of_view: Some(122.6),
            camera_mode: (),
            ..Default::default()
        };

        let j = serde_json::to_string(&settings).unwrap();
        println!("=======================");
        println!("\n\nTESTING {}", j);
        println!("=======================");

        // let root: VoiceRoot = serde_json::from_str(&data)?;
    }
}
