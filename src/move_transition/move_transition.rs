use crate::constants;
use crate::move_transition::models;
use crate::move_transition::private;
use crate::obs_filters;
use anyhow::Result;
use obws::Client as OBSClient;

// OBS_Client, Source, Scene
// could be the default info we need
// we need pass in settings
//
// and pass in a DurationSettings
//
// TODO: MoveTimingSettings

// Update and Trigger 3d Filter
pub async fn update_and_trigger_3d_filter<
    T: serde::Serialize
        + std::default::Default
        + obs_filters::three_d_transform::FilterName,
>(
    obs_client: &OBSClient,
    source: &str,
    duration_settings: models::DurationSettings,
    settings: T,
) -> Result<()> {
    let filter_name = settings.filter_name();
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: duration_settings.duration,
        easing_function: duration_settings.easing_function_index,
        easing_type: duration_settings.easing_type_index,
        settings,
        ..Default::default()
    };

    let move_transition_filter_name = format!("Move_{}", filter_name);

    Ok(private::update_filter_and_enable(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?)
}

pub async fn spin_source(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration_settings: models::DurationSettings,
) -> Result<()> {
    let filter_name =
        constants::THREE_D_TRANSITION_PERSPECTIVE_FILTER_NAME.to_string();

    let settings = obs_filters::three_d_transform::ThreeDTransformPerspective {
        rotation_z: Some(rotation_z),
        camera_mode: (),
        ..Default::default()
    };
    let new_settings = obs_filters::three_d_transform::MovePluginSettings {
        filter: filter_name.clone(),
        duration: duration_settings.duration,
        easing_function: duration_settings.easing_function_index,
        easing_type: duration_settings.easing_type_index,
        settings,
        ..Default::default()
    };

    let move_transition_filter_name = format!("Move_{}", filter_name);

    let _ = private::update_filter_and_enable(
        source,
        &move_transition_filter_name,
        new_settings,
        &obs_client,
    )
    .await?;
    Ok(())
}

pub async fn move_source_in_scene_x_and_y(
    obs_client: &OBSClient,
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration_settings: models::DurationSettings,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

    // Now we need a collapsed struct
    // I need something that takes in x, y and these values
    let new_settings = models::Coordinates {
        x: Some(x),
        y: Some(y),
    };

    // new_settings.duration = Some(duration);
    // new_settings.easing_type = Some(easing_type_index);
    // new_settings.easing_function = Some(easing_function_index);

    private::update_filter_and_enable(
        scene,
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}
