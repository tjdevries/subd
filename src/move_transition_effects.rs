// Spin and Stuff

use crate::move_transition;
use crate::obs;
use anyhow::Result;
use obws::responses::filters::SourceFilter;
use obws::Client as OBSClient;

pub async fn top_right(scene_item: &str, obs_client: &OBSClient) -> Result<()> {
    let base_settings = move_transition::fetch_source_settings(
        obs::DEFAULT_SCENE,
        &scene_item,
        &obs_client,
    )
    .await?;

    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_transition::move_with_move_source(
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}

pub async fn bottom_right(
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings = move_transition::fetch_source_settings(
        obs::DEFAULT_SCENE,
        &scene_item,
        &obs_client,
    )
    .await?;

    let new_settings =
        move_transition::custom_filter_settings(settings, 12.0, 878.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_transition::move_with_move_source(
        &filter_name,
        new_settings,
        &obs_client,
    )
    .await
}
