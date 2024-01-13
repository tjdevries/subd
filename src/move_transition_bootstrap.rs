use crate::move_transition;
use crate::obs;
use anyhow::Result;
use obws::Client as OBSClient;

// What's the difference between const and static
const MOVE_SOURCE_FILTER_KIND: &str = "move_source_filter";
const MOVE_VALUE_FILTER_KIND: &str = "move_value_filter";

pub async fn create_move_source_filter_from_file(
    scene: &str,
    source: &str,
    filter_name: &str,
    file_path: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let mut filter = move_transition::parse_json_into_struct(file_path);

    filter.source = Some(source.to_string());

    let new_filter = obws::requests::filters::Create {
        source: scene,
        filter: filter_name,
        kind: MOVE_SOURCE_FILTER_KIND,
        settings: Some(filter),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

pub async fn create_move_text_value_filter(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings = create_move_source_filter_settings(scene_item);
    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);

    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: MOVE_SOURCE_FILTER_KIND,
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

// ===========================================================

pub async fn create_move_source_filters(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let base_settings = create_move_source_filter_settings(scene_item);
    let new_settings =
        move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);

    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: MOVE_SOURCE_FILTER_KIND,
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

fn create_move_source_filter_settings(
    source: &str,
) -> move_transition::MoveSourceFilterSettings {
    let settings = move_transition::MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(300),
        bounds: Some(move_transition::Coordinates {
            x: Some(251.0),
            y: Some(234.0),
        }),
        scale: Some(move_transition::Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(move_transition::Coordinates {
            x: Some(1662.0),
            y: Some(13.0),
        }),
        crop: Some(move_transition::MoveSourceCropSetting {
            bottom: Some(0.0),
            left: Some(0.0),
            right: Some(0.0),
            top: Some(0.0),
        }),
        transform_text: Some("pos: x 1662.0 y 13.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0".to_string()),
        ..Default::default()
    };
    settings
}
