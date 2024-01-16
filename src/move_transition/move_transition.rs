use crate::constants;
use crate::move_transition::duration;
use crate::move_transition::models;
use crate::move_transition::move_value;
use crate::move_transition::private;
use crate::obs_filters;
use anyhow::Result;
use obws::Client as OBSClient;

// This uses the source passed in to find a filter that starts w/ Move_
// and ends with the source
pub async fn update_and_trigger_filter<
    T: serde::Serialize + std::default::Default,
>(
    obs_client: &OBSClient,
    source: &str,
    filter_name: &str,
    settings: T,
    duration_settings: duration::EasingDuration,
) -> Result<()> {
    let settings = move_value::Settings {
        target_filter: filter_name.to_string(),
        duration: duration_settings,
        settings,
        ..Default::default()
    };

    // ========================================
    let move_transition_filter_name = format!("Move_{}", filter_name);
    private::update_filter_and_enable(
        source,
        &move_transition_filter_name,
        settings,
        obs_client,
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
    let three_d_settings =
        obs_filters::three_d_transform::ThreeDTransformPerspective {
            rotation_z: Some(rotation_z),
            ..Default::default()
        };

    // ========================================
    let settings = move_value::Settings {
        target_filter: filter_name.clone(),
        duration: duration_settings,
        settings: three_d_settings,
        ..Default::default()
    };

    let move_transition_filter_name = format!("Move_{}", source);
    private::update_filter_and_enable(
        source,
        &move_transition_filter_name,
        settings,
        obs_client,
    )
    .await
}

pub async fn move_source_in_scene_x_and_y(
    obs_client: &OBSClient,
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    duration_settings: duration::EasingDuration,
) -> Result<()> {
    let settings = move_value::Settings::new(
        source.to_string(),
        models::Coordinates::new(Some(x), Some(y)),
        duration_settings,
    );

    // ========================================
    let move_transition_filter_name = format!("Move_{}", source);
    private::update_filter_and_enable(
        scene,
        &move_transition_filter_name,
        settings,
        obs_client,
    )
    .await
}
