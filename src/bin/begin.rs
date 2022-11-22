#![allow(dead_code)]

use obws::responses::filters::SourceFilter;
// use obws::client::Filters;
// use obws::requests::filters;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use std::time::Duration;
use std::{fs, thread};
// use std::path::Path;

// use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

// use rand::thread_rng as rng;

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

// use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

use obws::requests::scene_items::{
    Position, Scale, SceneItemTransform, SetTransform,
};
// use obws::requests::scene_items::SetTransform;
use obws::{client, Client as OBSClient};

use server::commands;
use server::users;
use subd_types::Event;
use tokio::sync::broadcast;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

const DEBUG: bool = false;
const STREAM_FX_FILTER: &str = "3D Transform";

const SINGLE_SETTING_VALUE_TYPE: u32 = 0;
const MULTI_SETTING_VALUE_TYPE: u32 = 1;

fn camera_type_config() -> HashMap<&'static str, i32> {
    let mut camera_types_per_filter = HashMap::new();
    camera_types_per_filter.insert("Corners.TopLeft.X", 2);

    camera_types_per_filter.insert("Corners.BottomLeft.Y", 0);
    camera_types_per_filter.insert("Corners.TopLeft.X", 0);
    camera_types_per_filter.insert("Corners.TopLeft.Y", 0);
    camera_types_per_filter.insert("Filter.Rotation.Z", 0);
    camera_types_per_filter.insert("Filter.Shear.X", 0);
    camera_types_per_filter.insert("Filter.Transform.Rotation.Z", 0);
    camera_types_per_filter.insert("Rotation.X", 0);
    camera_types_per_filter.insert("Rotation.Y", 0);
    camera_types_per_filter.insert("Rotation.Z", 0);

    // Come Back to Skew
    // camera_types_per_filter.insert("Shear.X", 0);
    // camera_types_per_filter.insert("Skew.X", 0);

    // This is 1
    camera_types_per_filter.insert("Position.X", 1);
    camera_types_per_filter.insert("Position.Y", 1);
    // camera_types_per_filter.insert("Rotation.X", 1);
    // camera_types_per_filter.insert("Rotation.Y", 1);
    // camera_types_per_filter.insert("Rotation.Z", 1);
    camera_types_per_filter.insert("Scale.X", 1);
    camera_types_per_filter.insert("Scale.Y", 1);
    camera_types_per_filter.insert("Shear.X", 1);
    camera_types_per_filter.insert("Shear.Y", 1);

    return camera_types_per_filter;
}

async fn move_with_move_source(
    scene_item: &str,
    filter_name: &str,
    new_settings: MoveSourceFilterSettings,
    obs_client: &client::Client,
) -> Result<()> {
    // let new_settings = top_corner_filter_settings(scene_item);
    update_move_source_filters(
        "Primary",
        scene_item,
        filter_name,
        new_settings,
        &obs_client,
    )
    .await?;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &"Primary",
        filter: &filter_name,
        // filter: "Move_Source",
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}
// let mut camera_types_per_filter = HashMap::new();
async fn handle_twitch_chat(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    // Technically, this one just needs to be able to read chat
    // this client won't send anything to chat.
    let config = get_chat_config();
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);
    let twitch_username = subd_types::consts::get_twitch_broadcaster_username();

    client.join(twitch_username.to_owned()).unwrap();

    while let Some(message) = incoming_messages.recv().await {
        match message {
            ServerMessage::Privmsg(private) => {
                tx.send(Event::TwitchChatMessage(private))?;
            }
            _ => {}
        }
    }

    Ok(())
}

// ==============================================================================

async fn handle_twitch_msg(
    _tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    let config = get_chat_config();
    let (_, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);

    let twitch_username = subd_types::consts::get_twitch_bot_username();

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        let _badges = msg
            .badges
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        subd_db::create_twitch_user_chat(
            &mut conn,
            &msg.sender.id,
            &msg.sender.login,
        )
        .await?;
        subd_db::save_twitch_message(
            &mut conn,
            &msg.sender.id,
            &msg.message_text,
        )
        .await?;

        let user_id =
            subd_db::get_user_from_twitch_user(&mut conn, &msg.sender.id)
                .await?;
        let _user_roles =
            users::update_user_roles_once_per_day(&mut conn, &user_id, &msg)
                .await?;

        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let paths = fs::read_dir("./MP3s").unwrap();

        let mut mp3s: HashSet<String> = vec![].into_iter().collect();

        for path in paths {
            mp3s.insert(path.unwrap().path().display().to_string());
        }

        match splitmsg[0].as_str() {
            "!echo" => {
                let echo = commands::Echo::try_parse_from(&splitmsg);
                if let Ok(echo) = echo {
                    let _ = client
                        .say(twitch_username.clone(), echo.contents)
                        .await;
                }
            }
            _ => {
                for word in splitmsg {
                    let sanitized_word = word.as_str().to_lowercase();
                    let full_name = format!("./MP3s/{}.mp3", sanitized_word);

                    if mp3s.contains(&full_name) {
                        // Works for Arch Linux
                        let (_stream, stream_handle) =
                            get_output_stream("pulse");

                        // Works for Mac
                        // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();

                        let sink =
                            rodio::Sink::try_new(&stream_handle).unwrap();

                        let file = BufReader::new(
                            File::open(format!(
                                "./MP3s/{}.mp3",
                                sanitized_word
                            ))
                            .unwrap(),
                        );

                        // TODO: Is there someway to suppress output here
                        sink.append(
                            rodio::Decoder::new(BufReader::new(file)).unwrap(),
                        );

                        sink.sleep_until_end();
                    }
                }
            }
        };
    }
}

// TODO: probably handle errors here
async fn change_scene(obs_client: &client::Client, name: &str) -> Result<()> {
    obs_client.scenes().set_current_program_scene(&name).await?;
    Ok(())
}

// ==============================================================================

#[derive(Debug)]
pub struct Scene {
    id: i64,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollSettings {
    #[serde(rename = "speed_x")]
    speed_x: Option<f32>,

    #[serde(rename = "speed_y")]
    speed_y: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlurSetting {
    #[serde(rename = "Commit")]
    commit: Option<String>,

    #[serde(rename = "Filter.Blur.Size")]
    size: Option<f32>,

    #[serde(rename = "Filter.Blur.StepScale")]
    step_scale: Option<bool>,

    #[serde(rename = "Filter.Blur.StepType")]
    step_type: Option<String>,

    #[serde(rename = "Filter.Blur.Version")]
    version: Option<u64>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSingleValueSetting {
    #[serde(rename = "duration")]
    duration: Option<u32>,

    #[serde(rename = "filter")]
    filter: String,

    #[serde(rename = "setting_float")]
    setting_float: f32,

    #[serde(rename = "setting_float_max")]
    setting_float_max: f32,

    #[serde(rename = "setting_float_min")]
    setting_float_min: f32,

    #[serde(rename = "setting_name")]
    setting_name: String,

    #[serde(rename = "value_type")]
    value_type: u32,

    #[serde(rename = "Filter.Blur.Size")]
    filter_blur_size: Option<f32>,

    #[serde(rename = "move_value_type")]
    move_value_type: Option<u32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    glow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    glow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    shadow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    shadow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Outline")]
    outline: Option<bool>,

    #[serde(rename = "source")]
    source: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Coordinates {
    #[serde(rename = "x")]
    x: Option<f32>,

    #[serde(rename = "y")]
    y: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceCropSetting {
    #[serde(rename = "bottom")]
    bottom: Option<f32>,

    #[serde(rename = "left")]
    left: Option<f32>,

    #[serde(rename = "top")]
    top: Option<f32>,

    #[serde(rename = "right")]
    right: Option<f32>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveSourceFilterSettings {
    crop: Option<MoveSourceCropSetting>,

    bounds: Option<Coordinates>,

    #[serde(rename = "pos")]
    position: Option<Coordinates>,

    scale: Option<Coordinates>,

    duration: Option<u64>,

    source: Option<String>,

    // How do we calculate the settings to this string
    //     "transform_text": "pos: x 83.0 y 763.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0",
    transform_text: Option<String>,
}

//     "duration": 3000,
//     "source": "kirbydance",
//
//     "filter": "",
//     "rot": 0.0,
//     "rot_sign": " ",
//     "setting_float": 0.0,
//     "setting_float_max": 0.0,
//     "setting_float_min": 0.0,
//     "setting_name": "",
//     "transform_text": "pos: x 83.0 y 763.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0",
//     "value_type": 0
//

// TODO: consider serde defaults???
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SDFEffectsSettings {
    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Alpha")]
    shadow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Alpha")]
    shadow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer")]
    glow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Alpha")]
    glow_outer_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Color")]
    outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Sharpness")]
    glow_outer_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Outer.Width")]
    glow_outer_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer")]
    shadow_outer: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Color")]
    shadow_outer_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Color")]
    shadow_inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    threshold: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner")]
    shadow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Alpha")]
    glow_inner_alpha: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner")]
    glow_inner: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Color")]
    inner_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Sharpness")]
    glow_inner_sharpness: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Glow.Inner.Width")]
    glow_inner_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Outline")]
    outline: Option<bool>,

    #[serde(rename = "Filter.SDFEffects.Outline.Color")]
    outline_color: Option<u64>,

    #[serde(rename = "Filter.SDFEffects.Outline.Width")]
    outline_width: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Maximum")]
    shadow_outer_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Maximum")]
    shadow_inner_range_max: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Range.Minimum")]
    shadow_inner_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Range.Minimum")]
    shadow_outer_range_min: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.Y")]
    shadow_inner_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Inner.Offset.X")]
    shadow_inner_offset_x: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.Shadow.Outer.Offset.Y")]
    shadow_outer_offset_y: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Scale")]
    sdf_scale: Option<f32>,

    #[serde(rename = "Filter.SDFEffects.SDF.Threshold")]
    sdf_threshold: Option<f32>,

    #[serde(rename = "Commit")]
    commit: Option<String>,

    #[serde(rename = "Version")]
    version: Option<u64>,
}

#[derive(Serialize, Debug)]
pub struct MoveOpacitySettings {
    #[serde(rename = "duration")]
    duration: i32,

    #[serde(rename = "filter")]
    filter: String,

    #[serde(rename = "setting_float")]
    setting_float: f32,

    #[serde(rename = "setting_float_max")]
    setting_float_max: f32,

    #[serde(rename = "setting_float_min")]
    setting_float_min: f32,

    #[serde(rename = "setting_name")]
    setting_name: String,

    #[serde(rename = "value_type")]
    value_type: i32,
}

// I think these might be different sometimes
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StreamFXSettings {
    #[serde(rename = "Camera.Mode")]
    camera_mode: Option<i32>,

    #[serde(rename = "Commit")]
    commit: String,

    #[serde(rename = "Position.X")]
    position_x: Option<f32>,

    #[serde(rename = "Position.Y")]
    position_y: Option<f32>,

    #[serde(rename = "Position.Z")]
    position_z: Option<f32>,

    #[serde(rename = "Rotation.X")]
    rotation_x: Option<f32>,

    #[serde(rename = "Rotation.Y")]
    rotation_y: Option<f32>,

    #[serde(rename = "Rotation.Z")]
    rotation_z: Option<f32>,

    #[serde(rename = "Version")]
    version: i64,
}

// "id": "streamfx-filter-transform",
// "mixers": 0,
// "monitoring_type": 0,
// "muted": false,
// "name": "YaBoi",
// "prev_ver": 469827586,
// "private_settings": {},
// "push-to-mute": false,
// "push-to-mute-delay": 0,
// "push-to-talk": false,
// "push-to-talk-delay": 0,
// "settings": {
//     "Camera.Mode": 1,
//     "Commit": "g0f114f56",
//     "Position.X": -0.01,
//     "Position.Y": -30.0,
//     "Position.Z": 0.02,
//     "Rotation.X": 43.93,
//     "Rotation.Y": -4.29,
//     "Rotation.Z": -2.14,
//     "Version": 51539607703
// },

async fn move_source(
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let id_search = obws::requests::scene_items::Id {
        scene: "Primary",
        source,
        ..Default::default()
    };
    let id = obs_client.scene_items().id(id_search).await?;

    // let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
    // let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);

    let new_position = Position {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        position: Some(new_position),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene: "Primary",
        item_id: id,
        transform: scene_transform,
    };
    match obs_client.scene_items().set_transform(set_transform).await {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok(())

    // .expect("Failed to Transform Scene Position")
}

async fn update_move_source_filters(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    new_settings: MoveSourceFilterSettings,
    obs_client: &OBSClient,
) -> Result<()> {
    // let new_settings = bottom_corner_filter_settings(scene_item);

    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(false),
    };
    obs_client.filters().set_settings(new_filter).await?;

    Ok(())
}

async fn create_move_source_filters(
    source: &str,
    scene_item: &str,
    filter_name: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let new_settings = top_corner_filter_settings(scene_item);

    let new_filter = obws::requests::filters::Create {
        source,
        filter: filter_name,
        kind: "move_source_filter",
        settings: Some(new_settings),
    };
    obs_client.filters().create(new_filter).await?;

    Ok(())
}

async fn create_outline_filter(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Outline";

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: "Outline",
        kind: "streamfx-filter-sdf-effects",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from("Outline"),
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

async fn create_blur_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Blur";

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: "Blur",
        kind: "streamfx-filter-blur",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from("Blur"),
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

fn top_corner_filter_settings(source: &str) -> MoveSourceFilterSettings {
    let position_x = 1662.0;
    let position_y = 13.0;

    let bounds_x = 251.0;
    let bounds_y = 243.0;

    let left = 0.0;
    let top = 0.0;
    let bottom = 0.0;
    let right = 0.0;

    let settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(Coordinates {
            x: Some(bounds_x),
            y: Some(bounds_y),
        }),
        scale: Some(Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(Coordinates {
            x: Some(position_x),
            y: Some(position_y),
        }),
        crop: Some(MoveSourceCropSetting {
            left: Some(0.0),
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
        }),
        transform_text: Some(format!("pos: x {position_x} y {position_y} rot: 0.0 bounds: x {bounds_x} y {bounds_y} crop: l {left} t {top} r {right} b {bottom}").to_string())
    };
    settings
}

fn custom_filter_settings(
    source: &str,
    duration: u64,
    x: f32,
    y: f32,
) -> MoveSourceFilterSettings {
    let position_x = x;
    let position_y = y;

    let bounds_x = 251.0;
    let bounds_y = 243.0;

    let left = 0.0;
    let top = 0.0;
    let bottom = 0.0;
    let right = 0.0;

    let settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(duration),
        bounds: Some(Coordinates {
            x: Some(bounds_x),
            y: Some(bounds_y),
        }),
        scale: Some(Coordinates {
            x: Some(2.0),
            y: Some(2.0),
        }),
        position: Some(Coordinates {
            x: Some(position_x),
            y: Some(position_y),
        }),
        crop: Some(MoveSourceCropSetting {
            left: Some(0.0),
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
        }),
        transform_text: Some(format!("pos: x {position_x} y {position_y} rot: 0.0 bounds: x {bounds_x} y {bounds_y} crop: l {left} t {top} r {right} b {bottom}").to_string())
    };
    settings
}

fn bottom_corner_filter_settings(source: &str) -> MoveSourceFilterSettings {
    let position_x = 12.0;
    let position_y = 878.0;

    let bounds_x = 251.0;
    let bounds_y = 243.0;

    let left = 0.0;
    let top = 0.0;
    let bottom = 0.0;
    let right = 0.0;

    let settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(Coordinates {
            x: Some(bounds_x),
            y: Some(bounds_y),
        }),
        scale: Some(Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(Coordinates {
            x: Some(position_x),
            y: Some(position_y),
        }),
        crop: Some(MoveSourceCropSetting {
            left: Some(0.0),
            top: Some(0.0),
            right: Some(0.0),
            bottom: Some(0.0),
        }),
        transform_text: Some(format!("pos: x {position_x} y {position_y} rot: 0.0 bounds: x {bounds_x} y {bounds_y} crop: l {left} t {top} r {right} b {bottom}").to_string())
    };
    settings
}

fn create_move_source_filter_settings(
    source: &str,
) -> MoveSourceFilterSettings {
    let settings = MoveSourceFilterSettings {
        source: Some(source.to_string()),
        duration: Some(4444),
        bounds: Some(Coordinates {
            x: Some(251.0),
            y: Some(234.0),
        }),
        scale: Some(Coordinates {
            x: Some(1.0),
            y: Some(1.0),
        }),
        position: Some(Coordinates {
            x: Some(1662.0),
            y: Some(13.0),
        }),
        crop: Some(MoveSourceCropSetting {
            bottom: Some(0.0),
            left: Some(0.0),
            right: Some(0.0),
            top: Some(0.0),
        }),
        transform_text: Some("pos: x 1662.0 y 13.0 rot: 0.0 bounds: x 251.000 y 234.000 crop: l 0 t 0 r 0 b 0".to_string())
    };
    settings
}

async fn create_scroll_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Scroll";

    let stream_fx_settings = StreamFXSettings {
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
    let new_settings = MoveSingleValueSetting {
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
async fn create_3d_transform_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Stream_FX";

    let stream_fx_settings = StreamFXSettings {
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: "3D Transform",
        kind: "streamfx-filter-transform",
        settings: Some(stream_fx_settings),
    };
    obs_client.filters().create(new_filter).await?;

    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(0),
        filter: String::from("3D Transform"),
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

async fn trigger_move_value_filter(
    source: &str,
    filter_setting_name: &str,
    splitmsg: &Vec<String>,
    obs_client: &OBSClient,
) -> Result<()> {
    // This is all to change the Camera Mode of 3D Transform
    // ===================================================================

    let camera_types_per_filter = camera_type_config();
    // I could hardcode this, but don't want to right now
    if !camera_types_per_filter.contains_key(&filter_setting_name) {
        return Ok(());
    }
    let camera_number = camera_types_per_filter[&filter_setting_name];
    // If it just unwraps
    // we fail on errors
    let filter_details =
        match obs_client.filters().get(&source, &"3D Transform").await {
            Ok(val) => val,
            Err(_err) => {
                return Ok(());
            }
        };
    // CRASHED!!!
    let mut new_settings =
        serde_json::from_value::<StreamFXSettings>(filter_details.settings)
            .unwrap();
    // Set Camera Mode on "3D Transform" Filter
    // so it matches the filter_setting_name
    new_settings.camera_mode = Some(camera_number);
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &"3D Transform",
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // ===================================================================

    let filter_value: f32 = if splitmsg.len() < 3 {
        0.0
    } else {
        splitmsg[2].trim().parse().unwrap_or(0.0)
    };

    let duration: u32 = if splitmsg.len() < 4 {
        3000
    } else {
        splitmsg[3].trim().parse().unwrap_or(3000)
    };

    update_and_trigger_move_value_filter(
        source,
        "Move_Stream_FX",
        filter_setting_name,
        filter_value,
        duration,
        SINGLE_SETTING_VALUE_TYPE,
        &obs_client,
    )
    .await?;
    Ok(())
}

async fn update_and_trigger_move_value_filter(
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

    let mut new_settings = serde_json::from_value::<MoveSingleValueSetting>(
        filter_details.settings,
    )
    .unwrap();

    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);

    new_settings.value_type = value_type;

    println!("\n!do New Settings: {:?}", new_settings);

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

// TODO: Update this function name to better
async fn handle_user_input(
    source: &str,
    filter_name: &str,
    splitmsg: &Vec<String>,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    let filter_setting_name = splitmsg[2].as_str();

    let filter_value: f32 = if splitmsg.len() < 4 {
        0.0
    } else {
        splitmsg[3].trim().parse().unwrap_or(0.0)
    };

    let duration: u32 = if splitmsg.len() < 5 {
        3000
    } else {
        splitmsg[4].trim().parse().unwrap_or(3000)
    };

    println!(
        "Handle User Input: Source {:?} | Filter Name: {:?} | Filter Setting Name: {:?} | Duration: {:?} | Value: {:?}",
        source, filter_name, filter_setting_name, duration, filter_value,
    );

    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    let mut new_settings = serde_json::from_value::<MoveSingleValueSetting>(
        filter_details.settings,
    )
    .unwrap();

    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);

    new_settings.value_type = value_type;

    println!("\n!do New Settings: {:?}", new_settings);

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

// Here you wait for OBS Events, that are commands to trigger OBS
async fn handle_obs_stuff(
    _tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;

    let config = get_chat_config();
    let (_, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);
    let twitch_username = subd_types::consts::get_twitch_bot_username();

    let obs_test_scene = "Primary";
    change_scene(&obs_client, &obs_test_scene).await?;

    let items = obs_client.scene_items().list(obs_test_scene).await?;
    if DEBUG {
        println!("Items: {:?}", items);
    }
    // let choosen_scene = Scene {
    //     id: 5,
    //     name: "BeginCam".to_string(),
    // };

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        // Split Message
        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // let = msg
        //     .message_text
        //     .split(" ")
        //     .map(|s| s.to_string())
        //     .collect::<Vec<String>>();

        // let first_char = splitmsg[0].chars().next().unwrap();
        // println!("First CHAR: {:?}", first_char);
        // let multiplier = first_char as u32;
        // let mut multiplier = multiplier as f32;

        // if (multiplier) < 100.0 {
        //     multiplier = 1.0;
        // } else {
        //     multiplier = -1.0;
        // }

        // Every single Word
        // for _word in splitmsg2 {
        //     let details = obs_client
        //         .scene_items()
        //         .transform(obs_test_scene, choosen_scene.id)
        //         .await?;
        //     let new_rot = details.rotation + (2.0 * multiplier);
        //     let scene_transform = SceneItemTransform {
        //         rotation: Some(new_rot),
        //         alignment: None,
        //         bounds: None,
        //         crop: None,
        //         scale: None,
        //         position: None,
        //     };
        //     let set_transform = SetTransform {
        //         scene: "Primary",
        //         item_id: choosen_scene.id,
        //         transform: scene_transform,
        //     };
        //     let _res = match obs_client
        //         .scene_items()
        //         .set_transform(set_transform)
        //         .await
        //     {
        //         Ok(_) => {
        //             println!("Successful Transform of Scene!");
        //         }
        //         Err(_) => {}
        //     };
        // }

        // let details = obs_client
        //     .scene_items()
        //     .transform(obs_test_scene, choosen_scene.id)
        //     .await?;
        // let _new_rot = details.rotation + 2.0;
        // if DEBUG {
        //     println!("Details {:?}", details);
        // }

        // let new_rot = details.rotation + 2.0;

        // // let rand_scale = rng.gen_range(0..100) as f32;
        // let new_scale_x = details.scale_x + (details.scale_x * 0.05);
        // let new_scale_y = details.scale_y + (details.scale_y * 0.05);
        // // let new_scale_x =
        // //     details.scale_x + (details.scale_x * (rand_scale / 100.0));
        // // let new_scale_y =
        // //     details.scale_y + (details.scale_y * (rand_scale / 100.0));
        // let new_scale = obws::requests::scene_items::Scale {
        //     x: Some(new_scale_x),
        //     y: Some(new_scale_y),
        // };

        // let new_x = details.position_x - (details.position_x * 0.005);
        // let new_y = details.position_y - (details.position_y * 0.02);
        // // let new_x =
        // //     details.position_x - (details.position_x * (rand_scale * 0.005));
        // // let new_y =
        // //     details.position_y - (details.position_y * (rand_scale * 0.02));
        // let new_position = obws::requests::scene_items::Position {
        //     x: Some(new_x),
        //     y: Some(new_y),
        // };
        // let scene_transform = SceneItemTransform {
        //     rotation: Some(new_rot),
        //     alignment: None,
        //     bounds: None,
        //     crop: None,
        //     scale: Some(new_scale),
        //     position: Some(new_position),
        // };
        // let set_transform = SetTransform {
        //     scene: "Primary",
        //     item_id: choosen_scene.id,
        //     transform: scene_transform,
        // };
        // let _res =
        //     match obs_client.scene_items().set_transform(set_transform).await {
        //         Ok(_) => {
        //             println!("I AM DUMB");
        //         }
        //         Err(_) => {}
        //     };

        // ===================================================

        // This is the same as holding the Super key on an Ergodox
        let super_key = obws::requests::hotkeys::KeyModifiers {
            shift: true,
            control: true,
            alt: true,
            command: true,
        };

        // let source = "shark";
        let source = "BeginCam";
        // let source = "Screen";

        // TODO: look up by effect setting name
        // let filter_name = "Move_SDF_Effects";
        // let filter_name = "Move_Stream_FX";

        // TODO: Implement these commands
        //   !3d
        //   !outline
        //   !scroll
        //   !blur
        //   !color

        let mut camera_types_per_filter = HashMap::new();
        camera_types_per_filter.insert("Corners.TopLeft.X", 2);

        camera_types_per_filter.insert("Corners.BottomLeft.Y", 0);
        camera_types_per_filter.insert("Corners.TopLeft.X", 0);
        camera_types_per_filter.insert("Corners.TopLeft.Y", 0);
        camera_types_per_filter.insert("Filter.Rotation.Z", 0);
        camera_types_per_filter.insert("Filter.Shear.X", 0);
        camera_types_per_filter.insert("Filter.Transform.Rotation.Z", 0);
        camera_types_per_filter.insert("Rotation.X", 0);
        camera_types_per_filter.insert("Rotation.Y", 0);
        camera_types_per_filter.insert("Rotation.Z", 0);

        // Come Back to Skew
        // camera_types_per_filter.insert("Shear.X", 0);
        // camera_types_per_filter.insert("Skew.X", 0);

        // This is 1
        camera_types_per_filter.insert("Position.X", 1);
        camera_types_per_filter.insert("Position.Y", 1);
        // camera_types_per_filter.insert("Rotation.X", 1);
        // camera_types_per_filter.insert("Rotation.Y", 1);
        // camera_types_per_filter.insert("Rotation.Z", 1);
        camera_types_per_filter.insert("Scale.X", 1);
        camera_types_per_filter.insert("Scale.Y", 1);
        camera_types_per_filter.insert("Shear.X", 1);
        camera_types_per_filter.insert("Shear.Y", 1);

        let default_stream_fx_filter_name = "Default_Stream_FX";
        let default_scroll_filter_name = "Default_Scroll";
        let default_blur_filter_name = "Default_Blur";
        let default_sdf_effects_filter_name = "Default_SDF_Effects";

        match splitmsg[0].as_str() {
            "!staff" => {
                let fake_input: Vec<String> = vec![
                    "!blur".to_string(),
                    "Filter.Blur.Size".to_string(),
                    "100".to_string(),
                ];

                // SINGLE_SETTING_VALUE_TYPE
                // MULTI_SETTING_VALUE_TYPE
                handle_user_input(
                    source,
                    "Move_Blur",
                    &fake_input,
                    2,
                    &obs_client,
                )
                .await?;

                // "duration": 3000,
                // "filter": "Scroll",
                // "move_value_type": 0,
                // "setting_float": 100.0,
                // "setting_float_max": 488.0,
                // "setting_float_min": 488.0,
                // "setting_name": "speed_x",
                // "value_type": 2

                // !scroll speed_x -115200
                let fake_input: Vec<String> = vec![
                    "!scroll".to_string(),
                    "speed_x".to_string(),
                    "-115200".to_string(),
                ];

                handle_user_input(
                    source,
                    "Move_Scroll",
                    &fake_input,
                    2,
                    &obs_client,
                )
                .await?;

                obs_client
                    .hotkeys()
                    .trigger_by_sequence("OBS_KEY_U", super_key)
                    .await?
            }
            "!all" => {
                // match splitmsg[1].as_str() {}
                // !all rot
                // How do we call something for all sources I wanna???
                let other_sources: Vec<String> = vec![
                    "gopher".to_string(),
                    "vibecat".to_string(),
                    "shark".to_string(),
                    "kirby".to_string(),
                ];

                let fake_split_msg = vec![
                    "!all".to_string(),
                    "fake_shit".to_string(),
                    "Rotation.Z".to_string(),
                    "3600".to_string(),
                ];

                for sub_scene in other_sources {
                    println!("sub_scene: {:?}", sub_scene);

                    handle_user_input(
                        source,
                        "Move_Stream_FX",
                        &fake_split_msg,
                        SINGLE_SETTING_VALUE_TYPE,
                        &obs_client,
                    )
                    .await?
                }

                // iterate through all and call
                // X Funtion
            }
            "!oo" => {
                let new_settings = SDFEffectsSettings {
                    glow_outer: Some(true),
                    shadow_outer: Some(true),
                    shadow_inner: Some(true),
                    glow_inner: Some(true),
                    outline: Some(true),

                    glow_inner_alpha: Some(100.0),
                    glow_inner_sharpness: Some(50.0),
                    glow_inner_width: Some(10.0),

                    glow_outer_alpha: Some(100.0),
                    glow_outer_sharpness: Some(50.0),
                    glow_outer_width: Some(10.0),

                    // outline_alpha: Some(100.0),
                    // outline_sharpness: Some(50.0),
                    outline_width: Some(10.0),
                    outline_color: Some(4294923775),

                    // outline_offset: Some(10.0),
                    //
                    shadow_inner_alpha: Some(100.0),
                    shadow_inner_offset_x: Some(0.0),
                    shadow_inner_offset_y: Some(0.0),
                    shadow_inner_range_max: Some(4.0),
                    shadow_inner_range_min: Some(0.0),
                    shadow_inner_color: Some(4278190335),

                    inner_color: Some(4278190335),
                    outer_color: Some(4294945280),

                    shadow_outer_alpha: Some(100.0),
                    shadow_outer_color: Some(4294945280),
                    shadow_outer_range_max: Some(4.0),
                    shadow_outer_range_min: Some(1.61),
                    shadow_outer_offset_y: Some(0.0),

                    scale: Some(1.0),
                    threshold: Some(50.0),

                    sdf_scale: Some(100.0),
                    sdf_threshold: Some(50.0),

                    commit: Some("g0f114f56".to_string()),
                    version: Some(51539607703),
                    ..Default::default()
                };

                let source = splitmsg[1].as_str();

                // rockerboo: inside the shaders for color its a float4 of R G B A
                // We need some color settings
                //
                // THIS IS THE COLOR
                // "Filter.SDFEffects.Glow.Outer.Color": 4294967295,
                // let settings_on = SDFEffectsSettings {
                //     glow_outer: Some(true),
                //     shadow_outer: Some(true),
                //     shadow_inner: Some(true),
                //     glow_inner: Some(true),
                //     outline: Some(true),
                //     // outer_color: Some(4294967295),
                //     outer_color: Some(4294902015),
                //     ..Default::default()
                // };

                let new_settings = obws::requests::filters::SetSettings {
                    source,
                    filter: "Outline",
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
            }
            "!reset" => {
                obs_client
                    .filters()
                    .remove(&source, &default_stream_fx_filter_name)
                    .await
                    .expect("Error Deleting Stream FX Default Filter");
                obs_client
                    .filters()
                    .remove(&source, &default_scroll_filter_name)
                    .await
                    .expect("Error Deleting Stream FX Default Filter");
                obs_client
                    .filters()
                    .remove(&source, &default_blur_filter_name)
                    .await
                    .expect("Error Deleting Stream FX Default Filter");
                obs_client
                    .filters()
                    .remove(&source, &default_sdf_effects_filter_name)
                    .await
                    .expect("Error Deleting Stream FX Default Filter");
            }
            "!norm" => {
                let source = if splitmsg.len() < 2 {
                    "begin"
                } else {
                    splitmsg[1].as_str()
                };

                println!("Attempting to Make: {source} normal!");

                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_stream_fx_filter_name,
                    enabled: true,
                };
                match obs_client.filters().set_enabled(filter_enabled).await {
                    Ok(_) => {}
                    Err(_) => continue,
                }
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_scroll_filter_name,
                    enabled: true,
                };
                match obs_client.filters().set_enabled(filter_enabled).await {
                    Ok(_) => {}
                    Err(_) => continue,
                }
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_blur_filter_name,
                    enabled: true,
                };
                match obs_client.filters().set_enabled(filter_enabled).await {
                    Ok(_) => {}
                    Err(_) => continue,
                }

                // This is ruining out life
                // we need a better set of defaults for the SDF
                // only should turn off filters
                //
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
            "!new_filters" => {
                let source = splitmsg[1].as_str();
                if source == "BeginCam" {
                    continue;
                }

                // let scroll_filter_name = "Move_Scroll";
                // let blur_filter_name = "Move_Blur";
                // let sdf_effects_filter_name = "Move_SDF_Effects";

                // Delete all the Filters For a Fresh Start
                let filters = obs_client.filters().list(source).await?;
                for filter in filters {
                    obs_client
                        .filters()
                        .remove(&source, &filter.name)
                        .await
                        .expect("Error Deleting Filter");
                }

                create_3d_transform_filters(source, &obs_client).await?;
                create_scroll_filters(source, &obs_client).await?;
                create_blur_filters(source, &obs_client).await?;
                create_outline_filter(source, &obs_client).await?;
            }

            "!shark" => {
                let source = "shark";
                let stream_fx_filter_name = "Move_Stream_FX";
                // let scroll_filter_name = "Move_Scroll";
                // let blur_filter_name = "Move_Blur";
                // let sdf_effects_filter_name = "Move_SDF_Effects";

                let stream_fx_settings = StreamFXSettings {
                    ..Default::default()
                };
                let new_filter = obws::requests::filters::Create {
                    source,
                    filter: "3D Transform",
                    kind: "streamfx-filter-transform",
                    settings: Some(stream_fx_settings),
                };
                obs_client.filters().create(new_filter).await?;

                let new_settings = MoveSingleValueSetting {
                    move_value_type: Some(0),
                    filter: String::from("3D Transform"),
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
            }
            "!top" => {
                let scene_item = splitmsg[1].as_str();
                let new_settings = top_corner_filter_settings(scene_item);
                let filter_name = format!("Move_Source_{}", scene_item);
                move_with_move_source(
                    &scene_item,
                    &filter_name,
                    new_settings,
                    &obs_client,
                )
                .await?;
            }
            "!bottom" => {
                let scene_item = splitmsg[1].as_str();
                let new_settings = bottom_corner_filter_settings(scene_item);
                let filter_name = format!("Move_Source_{}", scene_item);
                move_with_move_source(
                    &scene_item,
                    &filter_name,
                    new_settings,
                    &obs_client,
                )
                .await?;
            }
            "!update_move" => {
                let scene_item = splitmsg[1].as_str();
                let filter_name = format!("Move_Source_{}", scene_item);

                let new_settings = bottom_corner_filter_settings(scene_item);
                update_move_source_filters(
                    "Primary",
                    scene_item,
                    &filter_name,
                    new_settings,
                    &obs_client,
                )
                .await?;

                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &"Primary",
                    filter: "Move_Source",
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }
            "!create_move" => {
                // Should this create or should it modify a filter?
                // Should the name be more dynamic???
                let scene_item = splitmsg[1].as_str();
                create_move_source_filters(
                    "Primary",
                    scene_item,
                    "Move_Source",
                    &obs_client,
                )
                .await?;
            }

            "!create_move_filters" => {
                let other_sources: Vec<String> = vec![
                    // "begin".to_string(),
                    // "kirby".to_string(),
                    // "shark".to_string(),
                    // "gopher".to_string(),
                    // "vibecat".to_string(),
                ];
                for source in other_sources {
                    let filter_name = format!("Move_Source_{}", source);

                    create_move_source_filters(
                        "Primary",
                        &source,
                        &filter_name,
                        &obs_client,
                    )
                    .await?;
                }
            }
            "!create_defaults" => {
                let source = splitmsg[1].as_str();

                let new_settings = MoveSingleValueSetting {
                    move_value_type: Some(1),
                    filter: String::from("3D Transform"),
                    duration: Some(7000),
                    ..Default::default()
                };
                let new_filter = obws::requests::filters::Create {
                    source,
                    filter: default_stream_fx_filter_name,
                    kind: "move_value_filter",
                    settings: Some(new_settings),
                };
                obs_client.filters().create(new_filter).await?;

                // This is For Scroll
                let new_settings = MoveSingleValueSetting {
                    move_value_type: Some(1),
                    filter: String::from("Scroll"),
                    duration: Some(7000),
                    ..Default::default()
                };
                let new_filter = obws::requests::filters::Create {
                    source,
                    filter: default_scroll_filter_name,
                    kind: "move_value_filter",
                    settings: Some(new_settings),
                };
                obs_client.filters().create(new_filter).await?;

                // This is For Blur
                let new_settings = MoveSingleValueSetting {
                    move_value_type: Some(1),
                    filter: String::from("Blur"),
                    filter_blur_size: Some(1.0),
                    setting_float: 0.0,
                    duration: Some(7000),
                    ..Default::default()
                };
                let new_filter = obws::requests::filters::Create {
                    source,
                    filter: default_blur_filter_name,
                    kind: "move_value_filter",

                    settings: Some(new_settings),
                };
                obs_client.filters().create(new_filter).await?;

                // This is for SDF Effects
                let new_settings = MoveSingleValueSetting {
                    move_value_type: Some(1),
                    filter: String::from("Outline"),
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
                    filter: default_sdf_effects_filter_name,
                    kind: "move_value_filter",
                    settings: Some(new_settings),
                };
                obs_client.filters().create(new_filter).await?;
            }
            "!3d" => {
                let source = splitmsg[1].as_str();
                let filter_setting_name = splitmsg[2].as_str();

                if !camera_types_per_filter.contains_key(&filter_setting_name) {
                    continue;
                }

                let camera_number =
                    camera_types_per_filter[&filter_setting_name];

                let filter_details =
                    obs_client.filters().get(&source, &"3D Transform").await;

                let filt: SourceFilter;

                match filter_details {
                    Ok(val) => {
                        filt = val;
                    }
                    Err(_) => continue,
                }

                let mut new_settings =
                    serde_json::from_value::<StreamFXSettings>(filt.settings)
                        .unwrap();

                // resetting this Camera Mode
                new_settings.camera_mode = Some(camera_number);

                let new_settings = obws::requests::filters::SetSettings {
                    source: &source,
                    filter: &"3D Transform",
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;

                handle_user_input(
                    source,
                    "Move_Stream_FX",
                    &splitmsg,
                    SINGLE_SETTING_VALUE_TYPE,
                    &obs_client,
                )
                .await?;
            }

            // =========== //
            // DSL Section //
            // =========== //
            "!spin" => {
                let source = splitmsg[1].as_str();
                let filter_setting_name = "Rotation.Z";
                trigger_move_value_filter(
                    &source,
                    &filter_setting_name,
                    &splitmsg,
                    &obs_client,
                )
                .await?
            }

            "!spinx" => {
                let source = splitmsg[1].as_str();
                let filter_setting_name = "Rotation.X";
                trigger_move_value_filter(
                    &source,
                    &filter_setting_name,
                    &splitmsg,
                    &obs_client,
                )
                .await?
            }

            "!spiny" => {
                let source = splitmsg[1].as_str();
                let filter_setting_name = "Rotation.Y";
                trigger_move_value_filter(
                    &source,
                    &filter_setting_name,
                    &splitmsg,
                    &obs_client,
                )
                .await?
            }

            "!noblur" => {
                let source = splitmsg[1].as_str();

                update_and_trigger_move_value_filter(
                    source,
                    "Move_Blur",
                    "Filter.Blur.Size",
                    0.0,
                    5000,
                    2,
                    &obs_client,
                )
                .await?;
            }

            // !blur filter_name value duration
            "!blur" => {
                let source = splitmsg[1].as_str();

                update_and_trigger_move_value_filter(
                    source,
                    "Move_Blur",
                    "Filter.Blur.Size",
                    100.0,
                    5000,
                    2,
                    &obs_client,
                )
                .await?;
            }

            // !scrollx kirby 100
            // !scrolly kirby 100
            // !noscroll

            // !follow kirbydance
            //    this would take all the "moveable" sources
            //    and matches kirbydance's X and Y
            "!follow" => {
                let scene = "Primary";

                let leader = if splitmsg.len() < 2 {
                    "kirby"
                } else {
                    splitmsg[1].as_str()
                };
                let source = leader;

                let id_search = obws::requests::scene_items::Id {
                    scene,
                    source,
                    ..Default::default()
                };
                let id = obs_client.scene_items().id(id_search).await?;

                let other_sources: Vec<String> = vec![
                    "begin".to_string(),
                    "kirby".to_string(),
                    "shark".to_string(),
                    "gopher".to_string(),
                    "vibecat".to_string(),
                ];

                let settings =
                    match obs_client.scene_items().transform(scene, id).await {
                        Ok(val) => val,
                        Err(err) => {
                            println!(
                                "Error Fetching Transform Settings: {:?}",
                                err
                            );
                            let blank_transform =
                            obws::responses::scene_items::SceneItemTransform {
                                ..Default::default()
                            };
                            blank_transform
                        }
                    };

                let default_duration = 4444;
                let duration: u64 = if splitmsg.len() < 3 {
                    default_duration
                } else {
                    splitmsg[1].trim().parse().unwrap_or(default_duration)
                };

                for s in other_sources {
                    // Only if the source is not the leader!
                    if s != leader {
                        let new_settings = custom_filter_settings(
                            &s,
                            duration,
                            settings.position_x,
                            settings.position_y,
                        );
                        let filter_name = format!("Move_Source_{}", s);
                        move_with_move_source(
                            &s,
                            &filter_name,
                            new_settings,
                            &obs_client,
                        )
                        .await?;
                    }

                    // thread::sleep(Duration::from_millis(duration));
                }
            }

            "!source" => {
                let scene = "Primary";
                let source = splitmsg[1].as_str();

                let id_search = obws::requests::scene_items::Id {
                    scene,
                    source,
                    ..Default::default()
                };
                let id = obs_client.scene_items().id(id_search).await?;

                // Caused by:
                // invalid type: map, expected unit', src/bin/begin.rs:1495:5
                let settings =
                    match obs_client.scene_items().transform(scene, id).await {
                        Ok(val) => val,
                        Err(err) => {
                            println!(
                                "Error Fetching Transform Settings: {:?}",
                                err
                            );
                            let blank_transform =
                            obws::responses::scene_items::SceneItemTransform {
                                ..Default::default()
                            };
                            blank_transform
                        }
                    };

                println!("Source: {:?}", settings);
            }

            "!outline" => {
                handle_user_input(
                    source,
                    "Move_SDF_Effects",
                    &splitmsg,
                    SINGLE_SETTING_VALUE_TYPE,
                    &obs_client,
                )
                .await?;
            }

            "!scroll" => {
                let source = splitmsg[1].as_str();

                handle_user_input(
                    source,
                    "Move_Scroll",
                    &splitmsg,
                    2,
                    &obs_client,
                )
                .await?;
            }

            "!move" => {
                let source = splitmsg[1].as_str();

                let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
                let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);
                move_source(source, x, y, &obs_client).await?;
            }

            "!grow" => {
                let source = splitmsg[1].as_str();
                let id_search = obws::requests::scene_items::Id {
                    scene: "Primary",
                    source,
                    ..Default::default()
                };
                let id = obs_client.scene_items().id(id_search).await?;
                let new_scale = Scale {
                    x: Some(2.0),
                    y: Some(2.0),
                };

                let new_position = Position {
                    x: Some(2.0),
                    y: Some(2.0),
                };
                let scene_transform = SceneItemTransform {
                    scale: Some(new_scale),
                    position: Some(new_position),
                    // rotation: Some(None),
                    // alignment: None,
                    // bounds: None,
                    // crop: None,
                    // scale: None,
                    ..Default::default()
                };

                // I don't know the ID!!!!!!!
                let set_transform = SetTransform {
                    scene: "Primary",
                    item_id: id,
                    transform: scene_transform,
                };
                obs_client
                    .scene_items()
                    .set_transform(set_transform)
                    .await?
            }

            // ==========================================================================
            //
            "!yes_sdf" => {
                let settings_off = SDFEffectsSettings {
                    glow_outer: Some(true),
                    shadow_outer: Some(true),
                    shadow_inner: Some(true),
                    glow_inner: Some(true),
                    outline: Some(true),
                    ..Default::default()
                };

                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "Outline",
                    settings: settings_off,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
            }

            "!no_sdf" => {
                let settings_off = SDFEffectsSettings {
                    glow_outer: Some(false),
                    shadow_outer: Some(false),
                    shadow_inner: Some(false),
                    glow_inner: Some(false),
                    outline: Some(false),
                    ..Default::default()
                };

                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "Outline",
                    settings: settings_off,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
            }

            "!fs" => {
                println!("Trying to Read Filters");
                let filters = obs_client.filters().list("BeginCam").await?;
                println!("Filters {:?}", filters);
                client
                    .say(twitch_username.clone(), format!("{:?}", filters))
                    .await?;
            }
            "!filter" => {
                let (_command, words) =
                    msg.message_text.split_once(" ").unwrap();

                println!("Finding Filter Details {:?}", words);

                let filter_details = match obs_client
                    .filters()
                    .get("BeginCam", words)
                    .await
                {
                    Ok(details) => details,
                    Err(_) => {
                        println!("Error Fetching Filter Details: {:?}", words);
                        continue;
                    }
                };

                println!("Filter Details {:?}", filter_details);

                client
                    .say(
                        twitch_username.clone(),
                        format!("{:?}", filter_details),
                    )
                    .await?;
            }
            "!rand" => {
                // Oh it fails here!!!
                let amount = splitmsg[1].as_str();

                println!("Attempting!!!!");

                // how do I handle that
                let float_amount = match amount.parse::<f32>() {
                    Ok(val) => val,
                    Err(_) => {
                        println!("Error Parsing User Rand val");
                        continue;
                    }
                };

                // How do I convert this amount to float
                // Now I need to use this
                let settings: StreamFXSettings = StreamFXSettings {
                    camera_mode: Some(1),
                    commit: "g0f114f56".to_string(),
                    position_x: Some(-0.009999999776482582),
                    position_y: Some(float_amount),
                    position_z: Some(0.019999999552965164),
                    rotation_x: Some(243.92999267578125),
                    rotation_y: Some(-4.289999961853027),
                    rotation_z: Some(-2.140000104904175),
                    version: 51539607703,
                };
                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: STREAM_FX_FILTER,
                    // filter: "3D Transform",
                    settings,
                    overlay: None,
                };

                // This doesn't do anything????
                // So this is the call of it ???
                obs_client.filters().set_settings(new_settings).await?;
                // .unwrap();
            }

            // Rename These Commands
            "!chat" => {
                obs_client
                    .hotkeys()
                    .trigger_by_sequence("OBS_KEY_L", super_key)
                    .await?
            }
            "!code" => {
                obs_client
                    .hotkeys()
                    .trigger_by_sequence("OBS_KEY_H", super_key)
                    .await?
            }

            "!Primary" => {
                let obs_test_scene = "Primary";
                obs_client
                    .scenes()
                    .set_current_program_scene(&obs_test_scene)
                    .await?;
            }
            _ => {}
        }
    }
}

// ==============================================================================

// https://stackoverflow.com/questions/71468954/rust-rodio-get-a-list-of-outputdevices

fn list_host_devices() {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        println!(" # Device : {}", dev_name);
    }
}

fn get_output_stream(device_name: &str) -> (OutputStream, OutputStreamHandle) {
    let host = cpal::default_host();
    let devices = host.output_devices().unwrap();

    let (mut _stream, mut stream_handle) = OutputStream::try_default().unwrap();
    for device in devices {
        let dev: rodio::Device = device.into();
        let dev_name: String = dev.name().unwrap();
        if dev_name == device_name {
            println!("Device found: {}", dev_name);
            (_stream, stream_handle) =
                OutputStream::try_from_device(&dev).unwrap();
        }
    }
    return (_stream, stream_handle);
}

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

async fn say<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    client: &TwitchIRCClient<T, L>,
    msg: impl Into<String>,
) -> Result<()> {
    let twitch_username = subd_types::consts::get_twitch_broadcaster_username();
    client.say(twitch_username.to_string(), msg.into()).await?;
    Ok(())
}

// ==========================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::new("chat=debug,server=debug"))
        .without_time()
        .with_target(false)
        .finish()
        .init();

    {
        use rustrict::{add_word, Type};

        // You must take care not to call these when the crate is being
        // used in any other way (to avoid concurrent mutation).
        unsafe {
            add_word(format!("vs{}", "code").as_str(), Type::PROFANE);
            add_word("vsc*de", Type::SAFE);
        }
    }

    let mut channels = vec![];
    let (base_tx, _) = broadcast::channel::<Event>(256);

    macro_rules! makechan {
        // If it has (tx, rx) as signature, we can just do this
        ($handle_func:ident) => {{
            let (new_tx, new_rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move {
                $handle_func(new_tx, new_rx)
                    .await
                    .expect("this should work")
            }));
        }};

        (|$new_tx:ident, $new_rx:ident| $impl:block) => {{
            let ($new_tx, $new_rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move { $impl }));
        }};
    }

    makechan!(handle_twitch_chat);
    makechan!(handle_twitch_msg);
    makechan!(handle_obs_stuff);

    for c in channels {
        c.await?;
    }

    Ok(())
}
