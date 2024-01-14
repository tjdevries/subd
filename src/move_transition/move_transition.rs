use crate::constants;
use crate::move_transition::models;
use crate::move_transition::move_transition_bootstrap;
use crate::move_transition::private;
use crate::obs_filters;
use anyhow::Result;
use obws::Client as OBSClient;

// Why does this need to be MovePluginSetting???
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
    let _ = obs_client.filters().set_settings(settings).await?;
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

// Come back and replace w/
// MoveTimingSettings

// Update and Trigger 3d Filter
pub async fn update_and_trigger_3d_filter<
    T: serde::Serialize
        + std::default::Default
        + obs_filters::three_d_transform::FilterName,
>(
    obs_client: &OBSClient,
    source: &str,

    // TODO: these should be a struct
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
    Ok(update_and_trigger_move_values_filter(
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
    duration: u64,
    easing_function_index: Option<i32>,
    easing_type_index: Option<i32>,
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

pub async fn move_source_in_scene_x_and_y(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    _duration: u64,
    _easing_function_index: i32,
    _easing_type_index: i32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_name = format!("Move_{}", source);

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
