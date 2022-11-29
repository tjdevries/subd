use crate::move_transition;
use crate::sdf_effects;
use crate::stream_fx;
use anyhow::Result;
use obws;
use obws::requests::scene_items::{
    Position, Scale, SceneItemTransform, SetTransform,
};
use obws::responses::filters::SourceFilter;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

const DEFAULT_SCENE: &str = "Primary";
const MEME_SCENE: &str = "memes";
const DEFAULT_SOURCE: &str = "begin";

// Figure out what the name of this should really be
// const MULTI_SETTING_VALUE_TYPE: u32 = 1;
const SINGLE_SETTING_VALUE_TYPE: u32 = 0;

const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
const DEFAULT_SCROLL_FILTER_NAME: &str = "Default_Scroll";
const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";

const DEFAULT_SDF_EFFECTS_FILTER_NAME: &str = "Default_SDF_Effects";

const STREAM_FX_INTERNAL_FILTER_NAME: &str = "streamfx-filter-transform";
const MOVE_VALUE_INTERNAL_FILTER_NAME: &str = "move_value_filter";

// This ain't the name
const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D Transform";
const SDF_EFFECTS_FILTER_NAME: &str = "Outline";
const BLUR_FILTER_NAME: &str = "Blur";

// Constants to Extract:
// kind: "move_value_filter",
// kind: "streamfx-filter-blur",
// let stream_fx_filter_name = "Move_Blur";
// let stream_fx_filter_name = "Move_Scroll";
// filter: "Scroll",
// kind: "scroll_filter",

const SUPER_KEY: obws::requests::hotkeys::KeyModifiers =
    obws::requests::hotkeys::KeyModifiers {
        shift: true,
        control: true,
        alt: true,
        command: true,
    };

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollSettings {
    #[serde(rename = "speed_x")]
    pub speed_x: Option<f32>,

    #[serde(rename = "speed_y")]
    pub speed_y: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlurSetting {
    #[serde(rename = "Commit")]
    pub commit: Option<String>,

    #[serde(rename = "Filter.Blur.Size")]
    pub size: Option<f32>,

    #[serde(rename = "Filter.Blur.StepScale")]
    pub step_scale: Option<bool>,

    #[serde(rename = "Filter.Blur.StepType")]
    pub step_type: Option<String>,

    #[serde(rename = "Filter.Blur.Version")]
    pub version: Option<u64>,
}

// =========================== //
// 3D Transform Based Filters  //
// =========================== //

pub async fn default_ortho(
    source: &str,
    _duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // Change the underlying 3D Transform Filter
    let new_settings = stream_fx::StreamFXOrthographic {
        ..Default::default()
    };

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "3D_Orthographic",
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    Ok(())
}

pub async fn trigger_ortho(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let move_transition_filter_name = format!("Move_{}", filter_name);

    let filter_details = obs_client.filters().get(&source, &filter_name).await;

    let filt: SourceFilter = match filter_details {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let new_settings = match serde_json::from_value::<stream_fx::StreamFXSettings>(
        filt.settings,
    ) {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    _ = handle_user_input(
        source,
        &move_transition_filter_name,
        filter_setting_name,
        filter_value,
        duration,
        SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await;
    Ok(())
}

// TODO: This needs some heavy refactoring
// This only affects 3D transforms
pub async fn trigger_3d(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types_per_filter = camera_type_config();
    // if !camera_types_per_filter.contains_key(&filter_setting_name) {
    //     continue;
    // }

    // THIS CRASHESSSSSSS
    // WE NEED TO LOOK UP
    let camera_number = camera_types_per_filter[&filter_setting_name];

    // Look up information about the current "3D Transform" filter
    // IS THIS FUCKED????
    let filter_details = obs_client
        .filters()
        .get(&source, THE_3D_TRANSFORM_FILTER_NAME)
        .await;

    // Does this leave early??????
    let filt: SourceFilter = match filter_details {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    // TODO: Explore we are seeing this:
    // Error With New Settings: Error("missing field `Commit`", line: 0, column: 0)
    // // IS THIS STILL HAPPENING???
    let mut new_settings = match serde_json::from_value::<
        stream_fx::StreamFXSettings,
    >(filt.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error With New Settings: {:?}", e);
            stream_fx::StreamFXSettings {
                ..Default::default()
            }
        }
    };

    // I think we also want to return though!!!!
    // and not continue on here

    // resetting this Camera Mode
    new_settings.camera_mode = Some(camera_number);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: THE_3D_TRANSFORM_FILTER_NAME,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    handle_user_input(
        source,
        "Move_Stream_FX",
        filter_setting_name,
        filter_value,
        duration,
        SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await
}

pub async fn spin(
    source: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let setting_name = match filter_setting_name {
        "spin" | "z" => "Rotation.Z",
        "spinx" | "x" => "Rotation.X",
        "spiny" | "y" => "Rotation.Y",
        _ => "Rotation.Z",
    };

    // So if we ?
    // we fuck up???
    match update_and_trigger_move_value_filter(
        source,
        THE_3D_TRANSFORM_FILTER_NAME,
        setting_name,
        filter_value,
        duration,
        2, // not sure if this is the right value
        &obs_client,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("Error Updating and Triggering Value in !spin {:?}", e);
        }
    }

    Ok(())
}

// =============================== //
// Updating and Triggering Filters //
// =============================== //

pub async fn update_and_trigger_move_value_filter(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    let mut new_settings = match serde_json::from_value::<
        move_transition::MoveSingleValueSetting,
    >(filter_details.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {:?}", e);
            move_transition::MoveSingleValueSetting {
                ..Default::default()
            }
        }
    };

    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);

    new_settings.value_type = value_type;

    // Update the Filter
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // Why is there a wait???
    thread::sleep(Duration::from_millis(100));
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}
// =============== //
// Scaling Sources //
// =============== //

pub async fn scale_source(
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    // If we can't find the id for the passed in source
    // we just return Ok
    //
    // Should we log an error here?
    //
    // is there a more Idiomatic pattern?
    let id = match find_id(MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let new_scale = Scale {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene: DEFAULT_SCENE,
        item_id: id,
        transform: scene_transform,
    };
    match obs_client.scene_items().set_transform(set_transform).await {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok(())
}

pub async fn scale(
    id: i64,
    new_scale: Scale,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene: DEFAULT_SCENE,
        item_id: id,
        transform: scene_transform,
    };
    obs_client
        .scene_items()
        .set_transform(set_transform)
        .await?;
    Ok(())
}

// ================ //
// Just Move Things //
// ================ //

// Move a Source using x, y and NO Filters
pub async fn move_source(
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(e) => {
            println!("Error Finding ID {:?} {:?}", source, e);
            return Ok(());
        }
    };

    let new_position = Position {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        position: Some(new_position),
        ..Default::default()
    };

    // TODO: figure out looking up Scene, based on Source
    let set_transform = SetTransform {
        scene: MEME_SCENE,
        item_id: id,
        transform: scene_transform,
    };
    match obs_client.scene_items().set_transform(set_transform).await {
        Ok(_) => {}
        Err(e) => {
            println!("Error Transforming Scene: {:?}", e)
        }
    }

    Ok(())
}

pub async fn top_right(scene_item: &str, obs_client: &OBSClient) -> Result<()> {
    let base_settings =
        fetch_source_settings(DEFAULT_SCENE, &scene_item, &obs_client).await?;

    let new_settings = custom_filter_settings(base_settings, 1662.0, 13.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_with_move_source(&filter_name, new_settings, &obs_client).await
}

pub async fn bottom_right(
    scene_item: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let settings =
        fetch_source_settings(DEFAULT_SCENE, &scene_item, &obs_client).await?;

    let new_settings = custom_filter_settings(settings, 12.0, 878.0);
    let filter_name = format!("Move_Source_{}", scene_item);
    move_with_move_source(&filter_name, new_settings, &obs_client).await
}

// ================= //
// Hide/Show Actions //
// ================= //

pub async fn show_source(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    set_enabled(scene, source, true, obs_client).await
}

async fn set_enabled_on_all_sources(
    scene: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    match obs_client.scene_items().list(scene).await {
        Ok(items) => {
            for item in items {
                match set_enabled(
                    scene,
                    &item.source_name,
                    enabled,
                    &obs_client,
                )
                .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        println!(
                            "Error SetEnabled State of source {:?} {:?}",
                            item.source_name, e
                        );
                    }
                }
            }
            return Ok(());
        }
        Err(e) => {
            println!("Error listing Scene Items for {:?} {:?}", scene, e);
            return Ok(());
        }
    }
}

pub async fn hide_sources(scene: &str, obs_client: &OBSClient) -> Result<()> {
    set_enabled_on_all_sources(scene, false, &obs_client).await
}

// ============= //
// Combo Actions //
// ============= //

pub async fn norm(source: &str, obs_client: &OBSClient) -> Result<()> {
    println!("Attempting to Make: {source} normal!");

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &DEFAULT_STREAM_FX_FILTER_NAME,
        enabled: true,
    };
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &DEFAULT_SCROLL_FILTER_NAME,
        enabled: true,
    };
    // This is not the way
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: &DEFAULT_BLUR_FILTER_NAME,
        enabled: true,
    };
    match obs_client.filters().set_enabled(filter_enabled).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }

    let id = match find_id(MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let new_scale = Scale {
        x: Some(1.0),
        y: Some(1.0),
    };
    match scale(id, new_scale, &obs_client).await {
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }

    Ok(())

    // This is ruining our life
    // we need a better set of defaults for the SDF only should turn off filters

    // let filter_enabled = obws::requests::filters::SetEnabled {
    //     source: &source,
    //     filter: &default_sdf_effects_filter_name,
    //     enabled: true,
    // };
    // match obs_client.filters().set_enabled(filter_enabled).await {
    //     Ok(_) => {}
    //     Err(_) => continue,
    // }
}

pub async fn staff(source: &str, obs_client: &OBSClient) -> Result<()> {
    // This should be something more abstract
    // // Like Blur Begin
    _ = update_and_trigger_move_value_filter(
        source,
        "Move_Blur",
        "Filter.Blur.Size",
        100.0,
        5000,
        2,
        &obs_client,
    )
    .await;

    // What are these doing here like this?
    let filter_name = "Move_Source";
    let filter_setting_name = "speed_x";
    let filter_value = -115200.0;
    let duration = 5000;
    handle_user_input(
        source,
        filter_name,
        filter_setting_name,
        filter_value,
        duration,
        2,
        &obs_client,
    )
    .await?;

    obs_client
        .hotkeys()
        .trigger_by_sequence("OBS_KEY_U", SUPER_KEY)
        .await?;
    Ok(())
}

// TODO: This needs some heavy refactoring
pub async fn follow(
    scene: &str,
    source: &str,
    leader: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    let untouchable_sources: Vec<String> = vec![
        "Screen".to_string(),
        "SBF_Condo".to_string(),
        "Top Screen".to_string(),
        "twitchchat".to_string(),
        "Emotes".to_string(),
        "TwitchAlerts".to_string(),
    ];

    let leader_settings =
        match obs_client.scene_items().transform(scene, id).await {
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

    let sources = obs_client.scene_items().list(DEFAULT_SCENE).await?;
    for s in sources {
        for bad_source in &untouchable_sources {
            if bad_source == &s.source_name {
                // Will this continue out of the whole function???
                continue;
            }
        }
        let base_settings = match fetch_source_settings(
            DEFAULT_SCENE,
            &s.source_name,
            &obs_client,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                println!("Error fetching Source Settings: {:?}", err);
                continue;
            }
        };
        if s.source_name != leader {
            let new_settings = custom_filter_settings(
                base_settings,
                leader_settings.position_x,
                leader_settings.position_y,
            );
            let filter_name = format!("Move_Source_{}", s.source_name);
            _ = move_with_move_source(&filter_name, new_settings, &obs_client)
                .await;
        }
    }

    Ok(())
}

// ========================= //
// Bootstrap / Create Things //
// ========================= //

pub async fn create_move_source_filters(
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
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    Ok(())
}

pub async fn create_outline_filter(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Outline";

    // We look up Begin's Outline Settings
    let filter_details = match obs_client
        .filters()
        .get(DEFAULT_SOURCE, SDF_EFFECTS_FILTER_NAME)
        .await
    {
        Ok(val) => val,
        Err(_err) => {
            return Ok(());
        }
    };

    let new_settings =
        serde_json::from_value::<sdf_effects::SDFEffectsSettings>(
            filter_details.settings,
        )
        .unwrap();

    let new_filter = obws::requests::filters::Create {
        source,
        filter: SDF_EFFECTS_FILTER_NAME,
        kind: "streamfx-filter-sdf-effects",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // I think this is fucking shit up
    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(SDF_EFFECTS_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: stream_fx_filter_name,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_blur_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Blur";

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: BLUR_FILTER_NAME,
        kind: "streamfx-filter-blur",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from(BLUR_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: stream_fx_filter_name,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_scroll_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Scroll";

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: "Scroll",
        kind: "scroll_filter",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from("Scroll"),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: stream_fx_filter_name,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_split_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let camera_types = vec!["Orthographic", "Perspective", "CornerPin"];

    for (i, camera_type) in camera_types.iter().enumerate() {
        let filter_name = format!("3D_{}", camera_type);
        let stream_fx_settings = stream_fx::StreamFXSettings {
            camera_mode: Some(i as i32),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &filter_name,
            kind: STREAM_FX_INTERNAL_FILTER_NAME,
            settings: Some(stream_fx_settings),
        };
        obs_client.filters().create(new_filter).await?;

        // Create Move-Value for 3D Transform Filter
        let stream_fx_filter_name = format!("Move_3D_{}", camera_type);

        let new_settings = move_transition::MoveSingleValueSetting {
            move_value_type: Some(0),
            filter: String::from(filter_name),
            duration: Some(7000),
            ..Default::default()
        };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: &stream_fx_filter_name,
            kind: MOVE_VALUE_INTERNAL_FILTER_NAME,
            settings: Some(new_settings),
        };
        obs_client.filters().create(new_filter).await?;
    }

    Ok(())
}
pub async fn create_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Stream_FX";

    let stream_fx_settings = stream_fx::StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: THE_3D_TRANSFORM_FILTER_NAME,
        kind: "streamfx-filter-transform",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from(THE_3D_TRANSFORM_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: stream_fx_filter_name,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

pub async fn create_filters_for_source(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    println!("Creating Filters for Source: {}", source);

    let filters = match obs_client.filters().list(source).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
    };

    if source == DEFAULT_SOURCE {
        return Ok(());
    }

    for filter in filters {
        obs_client
            .filters()
            .remove(&source, &filter.name)
            .await
            .expect("Error Deleting Filter");
    }

    let filter_name = format!("Move_Source_Home_{}", source);
    create_move_source_filters(
        DEFAULT_SCENE,
        &source,
        &filter_name,
        &obs_client,
    )
    .await?;

    // We should seperate to it's own !chat command
    // create_split_3d_transform_filters(source, &obs_client).await?;
    create_3d_transform_filters(source, &obs_client).await?;
    create_scroll_filters(source, &obs_client).await?;
    create_blur_filters(source, &obs_client).await?;
    create_outline_filter(source, &obs_client).await?;

    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(THE_3D_TRANSFORM_FILTER_NAME),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: DEFAULT_STREAM_FX_FILTER_NAME,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // This is For Scroll
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from("Scroll"),
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: DEFAULT_SCROLL_FILTER_NAME,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // This is For Blur
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(BLUR_FILTER_NAME),
        filter_blur_size: Some(1.0),
        setting_float: 0.0,
        duration: Some(7000),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: DEFAULT_BLUR_FILTER_NAME,
        kind: "move_value_filter",

        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // This is for SDF Effects
    let new_settings = move_transition::MoveSingleValueSetting {
        move_value_type: Some(1),
        filter: String::from(SDF_EFFECTS_FILTER_NAME),
        duration: Some(7000),
        glow_inner: Some(false),
        glow_outer: Some(false),
        shadow_outer: Some(false),
        shadow_inner: Some(false),
        outline: Some(false),
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: DEFAULT_SDF_EFFECTS_FILTER_NAME,
        kind: "move_value_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    let filter_name = format!("Move_Source_{}", source);

    create_move_source_filters(
        DEFAULT_SCENE,
        &source,
        &filter_name,
        &obs_client,
    )
    .await?;

    Ok(())
}

// ========== //
// Fetch Info //
// ========== //

pub async fn print_source_info(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match find_id(MEME_SCENE, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => return Ok(()),
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

    println!("Source: {:?}", settings);
    Ok(())
}

pub async fn print_filter_info(
    source: &str,
    words: &str,
    obs_client: &OBSClient,
) -> Result<String> {
    println!("Finding Filter Details {:?}", words);

    let filter_details = match obs_client.filters().get(source, words).await {
        Ok(details) => details,
        Err(_) => {
            println!("Error Fetching Filter Details: {:?}", words);
            return Ok("".to_string());
        }
    };

    println!("Filter Details {:?}", filter_details);
    Ok(String::from(format!("{:?}", filter_details)))
}

// This just fetches settings around SDF Effects
// AND NOTHING ELSE!!!
pub async fn outline(source: &str, obs_client: &OBSClient) -> Result<()> {
    let filter_details = match obs_client
        .filters()
        .get(source, SDF_EFFECTS_FILTER_NAME)
        .await
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error Getting Filter Details: {:?}", e);
            return Ok(());
        }
    };

    // TODO: This could through an Error Here
    let new_settings =
        serde_json::from_value::<sdf_effects::SDFEffectsSettings>(
            filter_details.settings,
        )
        .unwrap();

    println!("\nFetched Settings: {:?}\n", new_settings);

    Ok(())
}

// ========================== //
// Update and Trigger Filters //
// ========================== //

pub async fn trigger_grow(
    source: &str,
    base_scale: &Scale,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    println!(
        "\n\t~~~ Attempting to Scale: {} | X: {:?} Y: {:?}",
        source, base_scale.x, base_scale.y
    );

    if source == "all" {
        let sources = obs_client.scene_items().list(DEFAULT_SCENE).await?;
        for source in sources {
            let new_scale = Scale {
                x: base_scale.x,
                y: base_scale.y,
            };
            let id = match find_id(MEME_SCENE, &source.source_name, &obs_client)
                .await
            {
                Ok(val) => val,
                Err(_) => return Ok(()),
            };

            if let Err(err) = scale(id, new_scale, &obs_client).await {
                println!("Error Finding ID: {}", err)
            };
        }
    } else {
        if let Err(err) = scale_source(&source, x, y, &obs_client).await {
            println!("Error Scaling Source: {}", err)
        };
    }
    Ok(())
}

pub async fn handle_user_input(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    duration: u32,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    println!(
        "Handle User Input: Source {:?} | Filter Name: {:?} | Filter Setting Name: {:?} | Duration: {:?} | Value: {:?}",
        source, filter_name, filter_setting_name, duration, filter_value,
    );

    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    let mut new_settings = serde_json::from_value::<
        move_transition::MoveSingleValueSetting,
    >(filter_details.settings)
    .unwrap();

    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);

    new_settings.value_type = value_type;

    println!("\nNew Settings: {:?}", new_settings);

    // Update the Filter
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;
    thread::sleep(Duration::from_millis(100));
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

// ======== //
// Hot Keys //
// ======== //

pub async fn trigger_hotkey(key: &str, obs_client: &OBSClient) -> Result<()> {
    _ = obs_client
        .hotkeys()
        .trigger_by_sequence(key, SUPER_KEY)
        .await;
    Ok(())
}

// ============= //
// Change Scenes //
// ============= //

pub async fn change_scene(obs_client: &obws::Client, name: &str) -> Result<()> {
    obs_client.scenes().set_current_program_scene(&name).await?;
    Ok(())
}

// =================== //
// Hide / Show Sources //
// =================== //

pub async fn set_enabled(
    scene: &str,
    source: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    match find_id(scene, source, &obs_client).await {
        Err(e) => {
            println!("Error finding ID for source {:?} {:?}", source, e)
        }
        Ok(id) => {
            let set_enabled: obws::requests::scene_items::SetEnabled =
                obws::requests::scene_items::SetEnabled {
                    enabled,
                    item_id: id,
                    scene,
                };

            match obs_client.scene_items().set_enabled(set_enabled).await {
                Err(e) => {
                    println!("Error Enabling Source: {:?} {:?}", source, e);
                }
                _ => (),
            }
        }
    };
    Ok(())
}

// ============== //
// Audio Settings //
// ============== //

pub async fn set_audio_status(
    _obs_conn: &obws::Client,
    _name: &str,
    _status: bool,
) -> Result<()> {
    // obs_conn.sources().(name, !status).await?;
    Ok(())
}

// ================= //
// Private Functions //
// ================= //

// These are the "Camera Type" we need for each of the filter types
// for the 3D Transform Effect
pub fn camera_type_config() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("Corners.TopLeft.X", 2),
        ("Corners.BottomLeft.Y", 0),
        ("Corners.TopLeft.X", 0),
        ("Corners.TopLeft.Y", 0),
        ("Filter.Rotation.Z", 0),
        ("Filter.Shear.X", 0),
        ("Filter.Transform.Rotation.Z", 0),
        ("Rotation.X", 0),
        ("Rotation.Y", 0),
        ("Rotation.Z", 0),
        ("Position.X", 1),
        ("Position.Y", 1),
        ("Scale.X", 1),
        ("Scale.Y", 1),
        ("Shear.X", 1),
        ("Shear.Y", 1),
    ])
}

async fn update_move_source_filters(
    source: &str,
    filter_name: &str,
    new_settings: move_transition::MoveSourceFilterSettings,
    obs_client: &OBSClient,
) -> Result<()> {
    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(false),
    };
    obs_client.filters().set_settings(new_filter).await?;

    Ok(())
}

fn create_move_source_filter_settings(
    source: &str,
) -> move_transition::MoveSourceFilterSettings {
    let settings = move_transition::MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
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

// This needs to take in Custom Filters
pub fn custom_filter_settings(
    mut base_settings: move_transition::MoveSourceFilterSettings,
    x: f32,
    y: f32,
) -> move_transition::MoveSourceFilterSettings {
    base_settings.position = Some(move_transition::Coordinates {
        x: Some(x),
        y: Some(y),
    });
    base_settings
}

async fn fetch_source_settings(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<move_transition::MoveSourceFilterSettings> {
    let id = match find_id(scene, source, &obs_client).await {
        Ok(val) => val,
        Err(_) => {
            return Ok(move_transition::MoveSourceFilterSettings {
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

    let new_settings = move_transition::MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(move_transition::Coordinates {
            x: Some(settings.bounds_width),
            y: Some(settings.bounds_height),
        }),
        scale: Some(move_transition::Coordinates {
            x: Some(settings.scale_x),
            y: Some(settings.scale_y),
        }),
        position: Some(move_transition::Coordinates {
            x: Some(settings.position_x),
            y: Some(settings.position_y),
        }),
        crop: Some(move_transition::MoveSourceCropSetting {
            left: Some(settings.crop_left as f32),
            right: Some(settings.crop_right as f32),
            bottom: Some(settings.crop_bottom as f32),
            top: Some(settings.crop_top as f32),
        }),
        transform_text: Some(transform_text.to_string()),
    };

    Ok(new_settings)
}

async fn find_id(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<i64, obws::Error> {
    let id_search = obws::requests::scene_items::Id {
        scene,
        source,
        ..Default::default()
    };

    obs_client.scene_items().id(id_search).await
}

// Updates the Move Filter with new Move Filter Settings
// Then calls the filter
async fn move_with_move_source(
    filter_name: &str,
    new_settings: move_transition::MoveSourceFilterSettings,
    obs_client: &obws::Client,
) -> Result<()> {
    update_move_source_filters(
        DEFAULT_SCENE,
        filter_name,
        new_settings,
        &obs_client,
    )
    .await?;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: DEFAULT_SCENE,
        filter: &filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}
