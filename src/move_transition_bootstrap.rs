use crate::move_transition;
use anyhow::Result;
use obws::Client as OBSClient;

pub async fn create_soundboard_text(obs_client: &OBSClient) -> Result<()> {
    let scene = "Characters";

    // let font_flags = obws::common::FontFlags{ }
    let font = obws::requests::custom::source_settings::Font {
        face: "Arial",
        size: 256,
        style: "Regular",
        ..Default::default()
    };

    // So these are fugazi???
    // I expect these colors to be something
    let color1 = rgb::RGBA::new(255, 0, 132, 0);
    let color2 = rgb::RGBA::new(0, 3, 255, 1);

    let text_settings =
        obws::requests::custom::source_settings::TextFt2SourceV2 {
            outline: true,
            drop_shadow: true,
            text: "SoundBoard!",
            color1,
            color2,
            font,
            custom_width: 5,
            log_lines: 5,
            word_wrap: false,
            ..Default::default() // We might want to experiment from file
        };

    let text_source_name = "Soundboard-Text";
    let _ = obs_client
        .inputs()
        .create(obws::requests::inputs::Create {
            scene,
            input: &text_source_name,
            kind: "text_ft2_source_v2",
            settings: Some(text_settings),
            enabled: Some(true),
        })
        .await;

    let filter_name = "TransformSoundBoard-text";
    let move_text_filter = move_transition::MoveTextFilter {
        setting_name: "text".to_string(),
        setting_text: "Ok NOW".to_string(),
        value_type: 5,
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source: &text_source_name,
        filter: &filter_name,
        kind: "move_value_filter",
        settings: Some(move_text_filter),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

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
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

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
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}
