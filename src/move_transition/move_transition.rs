use crate::move_transition::models;
use crate::move_transition::move_transition;
use crate::move_transition::move_transition_bootstrap;
use crate::move_transition::private;
use crate::obs_filters;
use anyhow::Result;
use obws::Client as OBSClient;
use std::thread;
use std::time::Duration;

pub async fn move_with_move_source<T: serde::Serialize>(
    scene: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    private::update_move_source_filters(
        scene,
        filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

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

    // Parse the settings into a MoveSingleValueSetting struct
    let mut new_settings = match serde_json::from_value::<
        models::MoveSingleValueSetting,
    >(filter_details.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {:?}", e);
            models::MoveSingleValueSetting {
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

// Update and Trigger 3d Filter
pub async fn update_and_trigger_3d_filter<
    T: serde::Serialize
        + std::default::Default
        + obs_filters::three_d_transform::FilterName,
>(
    obs_client: &OBSClient,
    source: &str,

    // These are an struct
    duration: u64,
    easing_function_index: Option<i32>,
    easing_type_index: Option<i32>,
    settings: T,
) -> Result<()> {
    let filter_name = settings.filter_name();
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: Some(duration as u32),
        easing_function: easing_function_index,
        easing_type: easing_type_index,
        settings,
        ..Default::default()
    };

    let move_transition_filter_name = format!("Move_{}", filter_name);
    let _ = update_and_trigger_move_values_filter(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

pub async fn update_and_trigger_move_values_filter<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: obs_filters::three_d_transform::MovePluginSettings<T>,
    obs_client: &OBSClient,
) -> Result<()> {
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
    Ok(())
}

pub async fn spin_source(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration: u64,
    easing_function_index: Option<i32>,
    easing_type_index: Option<i32>,
) -> Result<()> {
    let filter_name = "3D-Transform-Perspective".to_string();

    let settings = obs_filters::three_d_transform::ThreeDTransformPerspective {
        rotation_z: Some(rotation_z),
        camera_mode: (),
        ..Default::default()
    };
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: Some(duration as u32),
        easing_function: easing_function_index,
        easing_type: easing_type_index,
        settings,
        ..Default::default()
    };

    dbg!(&new_settings);

    let move_transition_filter_name = format!("Move_{}", filter_name);

    let _ = update_and_trigger_move_values_filter(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

pub async fn move_source_in_scene_x_and_y(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration: u64,
    easing_function_index: i32,
    easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    // TODO: These are incorrect
    let settings = move_transition_bootstrap::fetch_source_settings(
        scene,
        &source,
        &obs_client,
    )
    .await?;
    let mut new_settings =
        move_transition_bootstrap::custom_filter_settings(settings, x, y);

    new_settings.duration = Some(duration);
    new_settings.easing_type = Some(easing_type_index);
    new_settings.easing_function = Some(easing_function_index);

    move_transition::move_with_move_source(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}
