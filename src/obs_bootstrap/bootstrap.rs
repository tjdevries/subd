// use crate::constants;
// use crate::move_transition::models::MoveSingleValueSetting;
use anyhow::Result;
use obws;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};

// I don't know why it's 1 sometimes and it's 2 others
// RENAME THESE ONCE you figure what they are for
static MOVE_VALUE_TYPE_SINGLE: u32 = 1;
static MOVE_VALUE_TYPE_ZERO: u32 = 0;
static DEFAULT_DURATION: u32 = 7000;

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

// TODO: consider serde defaults???
// move into it's own SDF_Effects file???
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SDFEffectsSettings {
    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Alpha")]
    pub shadow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Alpha")]
    pub shadow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    pub glow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Alpha")]
    pub glow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Color")]
    pub outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Sharpness")]
    pub glow_outer_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Width")]
    pub glow_outer_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    pub shadow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Color")]
    pub shadow_outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Color")]
    pub shadow_inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    pub scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    pub threshold: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    pub shadow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Alpha")]
    pub glow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    pub glow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Color")]
    pub inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Sharpness")]
    pub glow_inner_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Width")]
    pub glow_inner_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Outline")]
    pub outline: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Outline.Color")]
    pub outline_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Outline.Width")]
    pub outline_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Maximum")]
    pub shadow_outer_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Maximum")]
    pub shadow_inner_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Minimum")]
    pub shadow_inner_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Minimum")]
    pub shadow_outer_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.Y")]
    pub shadow_inner_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.X")]
    pub shadow_inner_offset_x: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Offset.Y")]
    pub shadow_outer_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    pub sdf_scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    pub sdf_threshold: Option<f32>,

    #[serde(rename = "Commit")]
    pub commit: Option<String>,

    #[serde(rename = "Version")]
    pub version: Option<u64>,
}

// This just fetches settings around SDF Effects
// AND NOTHING ELSE!!!
pub async fn outline(source: &str, obs_client: &OBSClient) -> Result<()> {
    let filter_details = match obs_client
        .filters()
        .get(source, &subd_types::consts::get_sdf_effects_filter_name())
        .await
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error Getting Filter Details: {:?}", e);
            return Ok(());
        }
    };

    // TODO: This could through an Error Here
    let new_settings =
        serde_json::from_value::<SDFEffectsSettings>(filter_details.settings)
            .unwrap();

    println!("\nFetched Settings: {:?}\n", new_settings);

    Ok(())
}

pub async fn create_outline_filter(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name =
        subd_types::consts::get_move_outline_filter_name().to_string();

    // We look up Begin's Outline Settings
    let filter_details = match obs_client
        .filters()
        .get(
            &subd_types::consts::get_default_obs_source(),
            &subd_types::consts::get_sdf_effects_filter_name(),
        )
        .await
    {
        Ok(val) => val,
        Err(_err) => {
            return Ok(());
        }
    };

    let new_settings =
        serde_json::from_value::<SDFEffectsSettings>(filter_details.settings)
            .unwrap();

    let new_filter = obws::requests::filters::Create {
        source,
        filter: &subd_types::consts::get_sdf_effects_filter_name(),
        kind: &subd_types::consts::get_sdf_effects_internal_filter_name()
            .to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // I think this is fucking shit up
    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(MOVE_VALUE_TYPE_SINGLE),
        filter: String::from(subd_types::consts::get_sdf_effects_filter_name()),
        duration: Some(DEFAULT_DURATION),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &subd_types::consts::get_move_internal_filter_name().to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_blur_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = subd_types::consts::get_move_blur_filter_name();

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &subd_types::consts::get_blur_filter_name(),
        kind: &subd_types::consts::get_blur_internal_filter_name(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(subd_types::consts::get_blur_filter_name()),
        duration: Some(DEFAULT_DURATION),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &subd_types::consts::get_move_internal_filter_name(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_scroll_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name =
        subd_types::consts::get_move_scroll_filter_name();

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &subd_types::consts::get_scroll_filter_name(),
        kind: &subd_types::consts::get_scroll_internal_filter_name(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(&subd_types::consts::get_scroll_filter_name()),
        duration: Some(DEFAULT_DURATION),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &subd_types::consts::get_move_internal_filter_name().to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_split_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types = vec!["Orthographic", "Perspective", "CornerPin"];
    // let camera_types = vec![ "Perspective", "CornerPin"];

    for (i, camera_type) in camera_types.iter().enumerate() {
        let filter_name = format!("3D-Transform-{}", camera_type);
        let stream_fx_settings = StreamFXSettings {
            camera_mode: Some(i as i32),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &filter_name,
            kind: &subd_types::consts::get_stream_fx_internal_filter_name(),
            settings: Some(stream_fx_settings),
        };
        obs_client.filters().create(new_filter).await?;

        let stream_fx_filter_name = format!("Move_{}", filter_name.clone());

        // This is wrong
        let new_settings = MoveSingleValueSetting {
            move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
            filter: String::from(filter_name.clone()),
            duration: Some(DEFAULT_DURATION),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: &subd_types::consts::get_move_internal_filter_name(),
            settings: Some(new_settings),
        };
        let _ = obs_client.filters().create(new_filter).await;

        // Create Default Move-Value for 3D Transform Filter
        let stream_fx_filter_name = format!("Default_{}", filter_name.clone());

        let filter_name = format!("3D_{}", camera_type);
        let new_settings = MoveSingleValueSetting {
            move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
            filter: String::from(filter_name.clone()),
            duration: Some(3000),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: &subd_types::consts::get_move_internal_filter_name(),
            settings: Some(new_settings),
        };
        let _ = obs_client.filters().create(new_filter).await;
    }

    Ok(())
}

pub async fn create_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name =
        subd_types::consts::get_move_stream_fx_filter_name();

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &subd_types::consts::get_3d_transform_filter_name(),
        kind: &subd_types::consts::get_stream_fx_internal_filter_name(),
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(MOVE_VALUE_TYPE_ZERO),
        filter: String::from(subd_types::consts::get_3d_transform_filter_name()),
        duration: Some(DEFAULT_DURATION),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: &stream_fx_filter_name,
        kind: &subd_types::consts::get_move_internal_filter_name().to_string(),
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn remove_all_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let filters = match obs_client.filters().list(source).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    if source == subd_types::consts::get_default_obs_source() {
        return Ok(());
    }

    for filter in filters {
        obs_client
            .filters()
            .remove(&source, &filter.name)
            .await
            .expect("Error Deleting Filter");
    }
    Ok(())
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
