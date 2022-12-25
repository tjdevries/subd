use crate::move_transition;
use anyhow::Result;
use obws::Client as OBSClient;

pub const SINGLE_SETTING_VALUE_TYPE: u32 = 0;
pub const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D Transform";

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
        kind: "move_source_filter",
        settings: Some(filter),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

pub fn create_move_source_filter_settings(
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
        transform_text: Some("pos: x 1662.0 y 13.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0".to_string())
    };
    settings
}
