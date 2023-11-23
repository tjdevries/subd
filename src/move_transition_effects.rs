use crate::move_transition;
use crate::obs;
use crate::stream_fx;
use anyhow::Result;
use obws::responses::filters::SourceFilter;
use obws::Client as OBSClient;

pub async fn top_right(
    scene: &str,
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings =
        move_transition::fetch_source_settings(scene, &scene_item, &obs_client)
            .await?;

    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);
    let filter_name = format!("Move_{}", scene_item);
    // let filter_name = format!("Move_Source_{}", scene_item);
    println!("filter_name: {}", filter_name);

    // So what's the problem???
    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

pub async fn bottom_right(
    scene: &str,
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings =
        move_transition::fetch_source_settings(scene, &scene_item, &obs_client)
            .await?;

    let new_settings =
        move_transition::custom_filter_settings(settings, 12.0, 878.0);
    // let filter_name = format!("Move_Source_{}", scene_item);
    let filter_name = format!("Move_{}", scene_item);
    println!("filter_name: {}", filter_name);
    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

// SPIN
//
pub async fn spin(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // This feels like it belongs somewhere higher-up in the code
    let setting_name = match filter_setting_name {
        "spin" | "z" => "Rotation.Z",
        "spinx" | "x" => "Rotation.X",
        "spiny" | "y" => "Rotation.Y",
        _ => "Rotation.Z",
    };

    // TODO: fix
    match move_transition::update_and_trigger_move_value_filter(
        source,
        obs::THE_3D_TRANSFORM_FILTER_NAME,
        setting_name,
        filter_value,
        "",
        duration,
        2, // not sure if this is the right value | THIS NEEDS TO BE ABSTRACTED
        &obs_client,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("Error Updating and Triggering Value in !spin {:?}", e);
        }
    }

    Ok(())
}

// TODO: This needs some heavy refactoring
// This only affects 3D transforms
pub async fn trigger_3d(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types_per_filter = stream_fx::camera_type_config();

    let camera_number = camera_types_per_filter[&filter_setting_name];

    let filter_details = obs_client
        .filters()
        .get(&source, obs::THE_3D_TRANSFORM_FILTER_NAME)
        .await;

    let filt: SourceFilter = match filter_details {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let mut new_settings = match serde_json::from_value::<
        stream_fx::StreamFXSettings,
    >(filt.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };

    // Resetting this Camera Mode
    new_settings.camera_mode = Some(camera_number);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: obs::THE_3D_TRANSFORM_FILTER_NAME,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // TODO: Fix
    move_transition::update_and_trigger_move_value_filter(
        source,
        "Move_Stream_FX", // TODO Abstract this
        filter_setting_name,
        filter_value,
        "kjA,,jkjkk",
        duration,
        obs::SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await
}


// Filter name
// OG filter
// Move Filter
//
// Example: OG filter: 3D-Transform
//          Move Filter Move_3D-Transform
//
pub async fn trigger_move_value_3d_transform(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    camera_mode: &str,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    
    // let camera_types = vec!["Orthographic", "Perspective", "CornerPin"];
    // let camera_mode_index_raw = camera_types.iter().position(|&r| r == camera_mode).unwrap();
    // let camera_mode_index = camera_mode_index_raw.try_into().unwrap();

    let three_d_transform_filter_name = filter_name;
    // let three_d_transform_filter_name = format!("{}-{}", filter_name, camera_mode);
    let filter_settings = obs_client.filters().get(&source, &three_d_transform_filter_name).await;

    let filt: SourceFilter = match filter_settings {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };
    println!("\nOG 3D Transform Filter Settings: {:?}", filt);
    
    let new_settings = match serde_json::from_value::<stream_fx::StreamFXSettings>(
        filt.settings,
    ) {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };
    println!("\nNew 3D Transform Filter Settings: {:?}", new_settings);
    
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
    println!("MOVE {}", move_transition_filter_name);
    _ = move_transition::update_and_trigger_move_value_filter(
        source,
        &move_transition_filter_name,
        filter_setting_name,
        filter_value,
        &three_d_transform_filter_name,
        duration,
        obs::SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await;
    Ok(())
}
