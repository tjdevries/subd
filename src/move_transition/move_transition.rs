use crate::constants;
use crate::move_transition::duration;
use crate::move_transition::models;
use crate::move_transition::move_value;
use crate::move_transition::private;
use crate::obs_filters;
use anyhow::Result;
use obws::Client as OBSClient;

pub async fn update_and_trigger_single_value_filter<T: serde::Serialize>(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    settings: T,
) -> Result<()> {
    private::update_filter_and_enable(
        source,
        &filter_name,
        settings,
        obs_client,
    )
    .await
}

pub async fn update_and_trigger_filter<
    T: serde::Serialize + std::default::Default,
>(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    settings: T,
    duration_settings: duration::EasingDuration,
) -> Result<()> {
    let new_settings = move_value::Settings {
        filter: filter_name.to_string(),
        duration: duration_settings,
        settings,
        ..Default::default()
    };
    update_move_filter_and_enable_plus_rename(
        obs_client,
        source,
        &filter_name,
        new_settings,
    )
    .await
}

pub async fn spin_source(
    obs_client: &OBSClient,
    source: &str,
    rotation_z: f32,
    duration_settings: duration::EasingDuration,
) -> Result<()> {
    let filter_name =
        constants::THREE_D_TRANSITION_PERSPECTIVE_FILTER_NAME.to_string();
    let settings = obs_filters::three_d_transform::ThreeDTransformPerspective {
        rotation_z: Some(rotation_z),
        ..Default::default()
    };
    let new_settings = move_value::Settings {
        filter: filter_name.clone(),
        duration: duration_settings,
        settings,
        ..Default::default()
    };
    update_move_filter_and_enable_plus_rename(
        obs_client,
        source,
        &filter_name,
        new_settings,
    )
    .await
}

// We need another type here
pub async fn move_source_in_scene_x_and_y(
    obs_client: &OBSClient,
    _scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration_settings: duration::EasingDuration,
) -> Result<()> {
    let s = move_value::Settings {
        duration: duration_settings,
        settings: models::Coordinates {
            x: Some(x),
            y: Some(y),
        },
        ..Default::default()
    };
    update_move_filter_and_enable_plus_rename(obs_client, source, &source, s)
        .await
}

// ==========================================

async fn update_move_filter_and_enable_plus_rename<
    T: serde::Serialize + std::default::Default,
>(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    settings: T,
) -> Result<()> {
    let move_transition_filter_name = format!("Move_{}", filter_name);
    private::update_filter_and_enable(
        source,
        &move_transition_filter_name,
        settings,
        obs_client,
    )
    .await
}
