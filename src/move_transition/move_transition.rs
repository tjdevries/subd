use anyhow::Result;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

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

// ===================================================================================
// == Highest Level MOVE SOURCE
// ===================================================================================

pub async fn move_with_move_source<T: serde::Serialize>(
    scene: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    update_move_source_filters(scene, filter_name, new_settings, &obs_client)
        .await?;
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

// ===================================================================================
// == MOVE SOURCE ====================================================================
// ===================================================================================

pub async fn update_and_trigger_move_value_filter(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    target_filter_name: &str,
    duration: u32,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // Fetch the current settings of the filter we are going to update and trigger
    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    println!("------------------------");
    println!("\n\tOld Move Transition Settings: {:?}", filter_details);
    println!("------------------------");
    // Parse the settings into a MoveSingleValueSetting struct
    let mut new_settings = match serde_json::from_value::<MoveSingleValueSetting>(
        filter_details.settings,
    ) {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {:?}", e);
            MoveSingleValueSetting {
                ..Default::default()
            }
        }
    };

    println!("Target Filter Name: {}", target_filter_name);
    new_settings.filter = target_filter_name.to_string();

    // Update the settings based on what is passed into the function
    new_settings.source = Some(source.to_string());
    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);
    new_settings.value_type = value_type;
    new_settings.move_value_type = Some(value_type);

    println!("------------------------");
    println!("\n\n\tFinal Move Transition Settings: {:?}", new_settings);
    println!("------------------------");

    // Create a SetSettings struct & use it to update the OBS settings
    // TODO: Should this moved into the update_move_source_filters function?
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // Pause so the settings can take effect before triggering the filter
    // TODO: Extract out into variable
    thread::sleep(Duration::from_millis(400));

    // Trigger the filter
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

pub async fn update_and_trigger_move_values_filter_plus_cache(
    source: &str,
    filter_name: &str,
    mut new_settings: MoveMultipleValuesSetting,
    obs_client: &OBSClient,
) -> Result<()> {
    new_settings.move_value_type = 1;
    new_settings.value_type = 1;

    // This all needs to be customizable for names of filters
    // First we get all the values from the current Move filters
    let og_filter_settings =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;
    let j_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-j")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;
    let k_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-k")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;
    let l_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-l")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;

    // We then update all the Move filters
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let new_last_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-;",
        settings: l_filter_settings.settings,
        overlay: None,
    };
    let new_l_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-l",
        settings: k_filter_settings.settings,
        overlay: None,
    };
    let new_k_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-k",
        settings: j_filter_settings.settings,
        overlay: None,
    };
    let new_j_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-j",
        settings: og_filter_settings.settings,
        overlay: None,
    };

    let _ = obs_client.filters().set_settings(new_last_settings).await;
    let _ = obs_client.filters().set_settings(new_l_settings).await;
    let _ = obs_client.filters().set_settings(new_k_settings).await;
    let _ = obs_client.filters().set_settings(new_j_settings).await;

    // Trigger the main move filter filter
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    thread::sleep(Duration::from_millis(400));

    // Trigger the filter
    Ok(())
}

pub async fn update_and_trigger_move_values_filter(
    source: &str,
    filter_name: &str,
    mut new_settings: MoveMultipleValuesSetting,
    obs_client: &OBSClient,
) -> Result<()> {
    new_settings.move_value_type = 1;
    new_settings.value_type = 1;
    dbg!(&new_settings);
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: Some(true),
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    thread::sleep(Duration::from_millis(400));
    Ok(())
}

// ====================================================================
// == LOWER LEVEL???? =================================================
// ====================================================================

// This takes in settings and updates a filter
async fn update_move_source_filters<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &OBSClient,
) -> Result<()> {
    // What ever this serializes too, ain't right for Move Multiple Settings
    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(true),
    };
    obs_client.filters().set_settings(new_filter).await?;

    Ok(())
}
