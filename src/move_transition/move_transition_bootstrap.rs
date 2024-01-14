use crate::constants;
use crate::move_transition::models;
use crate::obs::obs_source;
use anyhow::Result;
use obws::Client as OBSClient;
use std::fs;

pub async fn create_move_source_filter_from_file(
    scene: &str,
    source: &str,
    filter_name: &str,
    file_path: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let mut filter = parse_json_into_struct(file_path);

    filter.source = Some(source.to_string());

    let new_filter = obws::requests::filters::Create {
        source: scene,
        filter: filter_name,
        kind: constants::MOVE_SOURCE_FILTER_KIND,
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
    let new_settings = custom_filter_settings(base_settings, 1662.0, 13.0);

    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: constants::MOVE_SOURCE_FILTER_KIND,
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

// ===========================================================

// pub async fn create_move_source_filters(
//     source: &str,
//     scene_item: &str,
//     filter_name: &str,
//     obs_client: &OBSClient,
// ) -> Result<()> {
//     let base_settings = create_move_source_filter_settings(scene_item);
//     let new_settings =
//         move_transition::custom_filter_settings(base_settings, 1662.0, 13.0);
//
//     let new_filter = obws::requests::filters::Create {
//         source,
//         filter: filter_name,
//         kind: MOVE_SOURCE_FILTER_KIND,
//         settings: Some(new_settings),
//     };
//     if let Err(err) = obs_client.filters().create(new_filter).await {
//         println!("Error Creating Filter: {filter_name} | {:?}", err);
//     };
//
//     Ok(())
// }

fn create_move_source_filter_settings(
    source: &str,
) -> models::MoveSourceFilterSettings {
    let settings = models::MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(300),
        bounds: Some(models::Coordinates {
            x: Some(251.0),
            y: Some(234.0),
        }),
        scale: Some(models::Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(models::Coordinates {
            x: Some(1662.0),
            y: Some(13.0),
        }),
        crop: Some(models::MoveSourceCropSetting {
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

// =======================================================================
// == Utilities ==========================================================
// =======================================================================

// This is a simple utility method
pub fn parse_json_into_struct(
    file_path: &str,
) -> models::MoveSourceFilterSettings {
    let contents = fs::read_to_string(file_path).expect("Can read file");

    let filter: models::MoveSourceFilterSettings =
        serde_json::from_str(&contents).unwrap();

    filter
}

pub fn custom_filter_settings(
    mut base_settings: models::MoveSourceFilterSettings,
    x: f32,
    y: f32,
) -> models::MoveSourceFilterSettings {
    base_settings.position = Some(models::Coordinates {
        x: Some(x),
        y: Some(y),
    });
    base_settings
}

// ===============================================================================
// == FETCHING ===================================================================
// ===============================================================================

pub async fn fetch_source_settings(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<models::MoveSourceFilterSettings> {
    let id = match obs_source::find_id(scene, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => {
            return Ok(models::MoveSourceFilterSettings {
                ..Default::default()
            })
        }
    };

    let settings = match obs_client.scene_items().transform(scene, id).await {
        Ok(val) => val,
        Err(err) => {
            println!("Error Fetching Transform Settings: {:?}", err);
            let blank_transform =
                obws::responses::scene_items::SceneItemTransform {
                    ..Default::default()
                };
            blank_transform
        }
    };

    let transform_text = format!(
        "pos: x {} y {} rot: 0.0 bounds: x {} y {} crop: l {} t {} r {} b {}",
        settings.position_x,
        settings.position_y,
        settings.bounds_width,
        settings.bounds_height,
        settings.crop_left,
        settings.crop_top,
        settings.crop_right,
        settings.crop_bottom
    );

    let new_settings = models::MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(models::Coordinates {
            x: Some(settings.bounds_width),
            y: Some(settings.bounds_height),
        }),
        scale: Some(models::Coordinates {
            x: Some(settings.scale_x),
            y: Some(settings.scale_y),
        }),
        position: Some(models::Coordinates {
            x: Some(settings.position_x),
            y: Some(settings.position_y),
        }),
        crop: Some(models::MoveSourceCropSetting {
            left: Some(settings.crop_left as f32),
            right: Some(settings.crop_right as f32),
            bottom: Some(settings.crop_bottom as f32),
            top: Some(settings.crop_top as f32),
        }),
        transform_text: Some(transform_text.to_string()),
        ..Default::default()
    };
    return Ok(new_settings);
}
