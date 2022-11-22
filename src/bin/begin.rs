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
        // let mut mp3s: HashSet<String> = vec![].into_iter().collect();

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
                // let messsages = HashSet::from(splitmsg);
                // HashSet::from() maybe
                // So Split this into a HashSet
                //
                // Can I convert a Vec<String> to Splitmsg
                for word in splitmsg {
                    // let word = splitmsg[0].as_str().to_lowercase();
                    // let full_name = format!("./MP3s/{}.mp3", sanitized_word);
                    let sanitized_word = word.as_str().to_lowercase();
                    let full_name = format!("./MP3s/{}.mp3", sanitized_word);

                    if mp3s.contains(&full_name) {
                        let (_stream, stream_handle) =
                            get_output_stream("pulse");

                        // let sink =
                        //     rodio::Sink::try_new(&stream_handle).unwrap();

                        // let file = std::fs::File::open("assets/music.mp3").unwrap();
                        // sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

                        // sink.sleep_until_end();
                        //         // This is incorrect
                        //         // let song_title = format!("./MP3s/{}.mp3", example);
                        //         // let rodioer = rodio::Decoder::new(BufReader::new(
                        //         //     Cursor::new(song_title),
                        //         // ))
                        //         // .unwrap();

                        //         // This works for Mac
                        //         // let (_stream, stream_handle) =
                        //         //     OutputStream::try_default().unwrap();

                        let file = BufReader::new(
                            File::open(format!(
                                "./MP3s/{}.mp3",
                                sanitized_word
                            ))
                            .unwrap(),
                        );

                        let source = Decoder::new(file).unwrap();

                        //         // We want to lower the volume
                        //         // Is this outputing the ALSA message????
                        stream_handle
                            .play_raw(source.convert_samples())
                            .expect("ok");

                        std::thread::sleep(std::time::Duration::from_secs(10));
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

    #[serde(rename = "Source")]
    source: Option<String>,
}

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

async fn create_move_source_filters(
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let stream_fx_filter_name = "Move_Source";

    // rockerboo: if let Ok(v) = { }
    // rockerboo: basically its a if statement to see if something is Ok vs error
    //
    // Create Move-Value for 3D Transform Filter
    let new_settings = MoveSingleValueSetting {
        move_value_type: Some(0),
        // filter: String::from("Blur"),
        duration: Some(7000),
        source: Some("kirbydance".to_string()),
        ..Default::default()
    };

    let new_filter = obws::requests::filters::Create {
        source,
        filter: stream_fx_filter_name,
        kind: "move_source_filter",
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

    // Handle User Input: Source "shark" | Filter Name: "Move_Stream_FX" | Duration: 10000 | Value: 3600.0

    // if let Ok(v) = { }
    // Should we pss in obs_client
    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => {
                Err(err)
                // continue
                // println!("Error Finding Filter Details: {:?}", err);
                // What should I do???
            }
        }?;

    // match filter_details {
    //     Ok(val) => {}
    //     Err(err) => break,
    // }

    // If this
    // println!("\n!do Filter Details: {:?}", filter_details);

    // Here is missing duration
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

    // That returns
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
                let source = splitmsg[1].as_str();
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_stream_fx_filter_name,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_scroll_filter_name,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_blur_filter_name,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &default_sdf_effects_filter_name,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
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
            "!create_move" => {
                create_move_source_filters("Primary", &obs_client).await?;
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

            // !follow kirbydance
            //    this would take all the "moveable" sources
            //    and matches kirbydance's X and Y
            "!follow" => {
                let scene = "Primary";
                let source = splitmsg[1].as_str();

                let id_search = obws::requests::scene_items::Id {
                    scene,
                    source,
                    ..Default::default()
                };
                let id = obs_client.scene_items().id(id_search).await?;

                let other_sources: Vec<String> = vec![
                    "gopher".to_string(),
                    "vibecat".to_string(),
                    "shark".to_string(),
                ];

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

                for s in other_sources {
                    move_source(
                        &s,
                        settings.position_x,
                        settings.position_y,
                        &obs_client,
                    )
                    .await?;
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

            // !blur filter_name value duration
            "!blur" => {
                let source = splitmsg[1].as_str();

                handle_user_input(
                    source,
                    "Move_Blur",
                    &splitmsg,
                    2,
                    &obs_client,
                )
                .await?;
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
