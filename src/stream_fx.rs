use crate::move_transition;
use crate::obs;
use anyhow::Result;
use obws::responses::filters::SourceFilter;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub async fn default_ortho(
    source: &str,
    _duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let new_settings = move_transition::default_orthographic_settings();

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "3D_Orthographic",
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    Ok(())
}

pub async fn trigger_ortho(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let move_transition_filter_name = format!("Move_{}", filter_name);

    let filter_details = obs_client.filters().get(&source, &filter_name).await;

    let filt: SourceFilter = match filter_details {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let new_settings =
        match serde_json::from_value::<StreamFXSettings>(filt.settings) {
            Ok(val) => val,
            Err(e) => {
                println!("Error With New Settings: {:?}", e);
                StreamFXSettings {
                    ..Default::default()
                }
            }
        };

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    _ = obs::handle_user_input(
        source,
        &move_transition_filter_name,
        filter_setting_name,
        filter_value,
        duration,
        obs::SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await;
    Ok(())
}

// ==========================================================================

// THIS IS FOR STREAM_FX
// These are the "Camera Type" we need for each of the filter types
// for the 3D Transform Effect
pub fn camera_type_config() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("Corners.TopLeft.X", 2),
        ("Corners.BottomLeft.Y", 0),
        ("Corners.TopLeft.X", 0),
        ("Corners.TopLeft.Y", 0),
        ("Filter.Rotation.Z", 0),
        ("Filter.Shear.X", 0),
        ("Filter.Transform.Rotation.Z", 0),
        ("Rotation.X", 0),
        ("Rotation.Y", 0),
        ("Rotation.Z", 0),
        ("Position.X", 1),
        ("Position.Y", 1),
        ("Scale.X", 1),
        ("Scale.Y", 1),
        ("Shear.X", 1),
        ("Shear.Y", 1),
    ])
}
