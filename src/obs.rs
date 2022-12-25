use crate::move_transition;
use crate::obs_source;
use anyhow::Result;
use obws;
use obws::Client as OBSClient;

use std::thread;
use std::time::Duration;

// TODO: We need to audit all of these
pub const DEFAULT_SCENE: &str = "Primary";
pub const MEME_SCENE: &str = "memes";

pub const SINGLE_SETTING_VALUE_TYPE: u32 = 0;

pub const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
pub const DEFAULT_SCROLL_FILTER_NAME: &str = "Default_Scroll";
pub const DEFAULT_SDF_EFFECTS_FILTER_NAME: &str = "Default_SDF_Effects";
pub const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";

pub const MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
pub const MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";
pub const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D Transform";
pub const SDF_EFFECTS_FILTER_NAME: &str = "Outline";

// This is for hotkeys
pub const SUPER_KEY: obws::requests::hotkeys::KeyModifiers =
    obws::requests::hotkeys::KeyModifiers {
        shift: true,
        control: true,
        alt: true,
        command: true,
    };

// ==============================================================

// TODO: What kinda trash name is this???
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

    // Does this need to be longer
    thread::sleep(Duration::from_millis(200));

    println!("Attempting to enable Filter: {} {}", source, filter_name);

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

// ==============================================================================

// ========== //
// Fetch Info //
// ========== //

pub async fn print_source_info(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = match obs_source::find_id(MEME_SCENE, source, &obs_client).await {
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

// =======================================================================================================

// ============= //
// Change Scenes //
// ============= //

pub async fn change_scene(obs_client: &obws::Client, name: &str) -> Result<()> {
    obs_client.scenes().set_current_program_scene(&name).await?;
    Ok(())
}

pub async fn find_scene(source: &str) -> Result<String> {
    let scene = match source {
        "begin" => DEFAULT_SCENE,
        _ => MEME_SCENE,
    };

    Ok(scene.to_string())
}

// =======================================================================

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
