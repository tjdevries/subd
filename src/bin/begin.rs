#![allow(dead_code)]

use obws::client::Filters;
use obws::requests::filters;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use std::fs;
// use std::path::Path;

// use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

// use rand::thread_rng as rng;

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

// use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

use obws::requests::scene_items::SceneItemTransform;
use obws::requests::scene_items::SetTransform;
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
pub struct MoveSingleValueSetting {
    #[serde(rename = "duration")]
    duration: u32,

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
}

// Filter Details: SourceFilter { enabled: true, index: 10, kind: "streamfx-filter-sdf-effects", name: "", settings: Object {"Commit": String("g0f114f56"), "Filter.SDFEffects.Glow.Inner": Bool(false), "Filter.SDFEffects.Glow.Inner.Color": Number(4286513322), "Filter.SDFEffects.Glow.Inner.Width": Number(9.14), "Filter.SDFEffects.Glow.Outer": Bool(false), "Filter.SDFEffects.Glow.Outer.Alpha": Number(0.0), "Filter.SDFEffects.Glow.Outer.Color": Number(4278255360), "Filter.SDFEffects.Glow.Outer.Sharpness": Number(67.73), "Filter.SDFEffects.Glow.Outer.Width": Number(16.0), "Filter.SDFEffects.Outline": Bool(true), "Filter.SDFEffects.Outline.Color": Number(4278255615), "Filter.SDFEffects.Outline.Sharpness": Number(55.48), "Filter.SDFEffects.Outline.Width": Number(0.0), "Filter.SDFEffects.SDF.Scale": Number(72.7), "Filter.SDFEffects.SDF.Threshold": Number(59.12), "Filter.SDFEffects.Shadow.Inner": Bool(false), "Filter.SDFEffects.Shadow.Inner.Color": Number(4286578432), "Filter.SDFEffects.Shadow.Inner.Offset.Y": Number(67.18), "Filter.SDFEffects.Shadow.Inner.Range.Minimum": Number(-1.95), "Filter.SDFEffects.Shadow.Outer": Bool(false), "Filter.SDFEffects.Shadow.Outer.Color": Number(4294923775), "Filter.SDFEffects.Shadow.Outer.Offset.X": Number(87.63), "Filter.SDFEffects.Shadow.Outer.Range.Maximum": Number(4.99), "Filter.SDFEffects.Shadow.Outer.Range.Minimum": Number(0.44), "Version": Number(51539607703), "glow_outer_width": Number(0.0)} }
//
// TODO: consider serde defaults???
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SDFEffectsSettings {
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
#[derive(Serialize, Debug, Default)]
pub struct StreamFXSettings {
    #[serde(rename = "Camera.Mode")]
    camera_mode: i32,
    #[serde(rename = "Commit")]
    commit: String,
    #[serde(rename = "Position.X")]
    position_x: f32,
    #[serde(rename = "Position.Y")]
    position_y: f32,
    #[serde(rename = "Position.Z")]
    position_z: f32,
    #[serde(rename = "Rotation.X")]
    rotation_x: f32,
    #[serde(rename = "Rotation.Y")]
    rotation_y: f32,
    #[serde(rename = "Rotation.Z")]
    rotation_z: f32,
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
    let choosen_scene = Scene {
        id: 5,
        name: "BeginCam".to_string(),
    };

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

        let splitmsg2 = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // rockerboo: chars().map(|char| char.to_digit()).collect()
        //0
        //thread 'tokio-runtime-worker' panicked at 'called `Option::unwrap()` on a `None` value', src/bin/begin.rs:332:62
        let first_char = splitmsg[0].chars().next().unwrap();
        println!("First CHAR: {:?}", first_char);
        let multiplier = first_char as u32;
        let mut multiplier = multiplier as f32;
        // let char_num = first_char as u32;
        // let maltipler = char_num as f32;
        // let char_num: f32 = (first_char).cast();

        // println!("New CHAR: {:?}", new_char);
        // let first_num = first_char.parse::<u32>().unwrap();

        // let first_char = splitmsg[0].chars().next().to_digit(10).unwrap();
        // splitmsg[0].chars().next().unwrap().to_digit(10).unwrap();

        // println!("\n\nFirst CHar: {:?}", first_char);

        // let multiplier = 1.0;
        if (multiplier) < 100.0 {
            multiplier = 1.0;
        } else {
            multiplier = -1.0;
        }

        // Every single Word
        for _word in splitmsg2 {
            let details = obs_client
                .scene_items()
                .transform(obs_test_scene, choosen_scene.id)
                .await?;
            let new_rot = details.rotation + (2.0 * multiplier);
            let scene_transform = SceneItemTransform {
                rotation: Some(new_rot),
                alignment: None,
                bounds: None,
                crop: None,
                scale: None,
                position: None,
            };
            let set_transform = SetTransform {
                scene: "Primary",
                item_id: choosen_scene.id,
                transform: scene_transform,
            };
            let _res = match obs_client
                .scene_items()
                .set_transform(set_transform)
                .await
            {
                Ok(_) => {
                    println!("Successful Transform of Scene!");
                }
                Err(_) => {}
            };
        }

        let details = obs_client
            .scene_items()
            .transform(obs_test_scene, choosen_scene.id)
            .await?;
        let _new_rot = details.rotation + 2.0;
        if DEBUG {
            println!("Details {:?}", details);
        }

        // cafce25: no gen_range will return a value of the range
        //
        // TODO: Move this out!!!
        // Update a Scene's Settings
        // let rand_rot = rng.gen_range(0..100) as f32;
        // e.g. `thread_rng().gen::<i32>()`, or cached locally, e.g.
        // let new_rot = details.rotation + (rng().gen_range(0..10) as f32);
        // rng.gen::<f32>();
        let new_rot = details.rotation + 2.0;

        // let rand_scale = rng.gen_range(0..100) as f32;
        let new_scale_x = details.scale_x + (details.scale_x * 0.05);
        let new_scale_y = details.scale_y + (details.scale_y * 0.05);
        // let new_scale_x =
        //     details.scale_x + (details.scale_x * (rand_scale / 100.0));
        // let new_scale_y =
        //     details.scale_y + (details.scale_y * (rand_scale / 100.0));
        let new_scale = obws::requests::scene_items::Scale {
            x: Some(new_scale_x),
            y: Some(new_scale_y),
        };

        let new_x = details.position_x - (details.position_x * 0.005);
        let new_y = details.position_y - (details.position_y * 0.02);
        // let new_x =
        //     details.position_x - (details.position_x * (rand_scale * 0.005));
        // let new_y =
        //     details.position_y - (details.position_y * (rand_scale * 0.02));
        let new_position = obws::requests::scene_items::Position {
            x: Some(new_x),
            y: Some(new_y),
        };
        let scene_transform = SceneItemTransform {
            rotation: Some(new_rot),
            alignment: None,
            bounds: None,
            crop: None,
            scale: Some(new_scale),
            position: Some(new_position),
        };
        let set_transform = SetTransform {
            scene: "Primary",
            item_id: choosen_scene.id,
            transform: scene_transform,
        };
        let _res =
            match obs_client.scene_items().set_transform(set_transform).await {
                Ok(_) => {
                    println!("I AM DUMB");
                }
                Err(_) => {}
            };

        // let filter_details =
        //     obs_client.filters().get("BeginCam", "Hot").await?;
        // println!("Hot Filter: {:?}\n\n", filter_details);
        // // Enable Filter
        // // This makes Begin Red
        // let filter_enabled = obws::requests::filters::SetEnabled {
        //     source: "BeginCam",
        //     filter: "Hot",
        //     enabled: !filter_details.enabled,
        // };
        // obs_client.filters().set_enabled(filter_enabled).await?;

        // Flip filters
        // Switch to Scenes
        // TODO: Update Filters

        // let filter_name = "WHA";

        // cafce25: if you use rand::seq::SliceRandom; you can options.choose(rng()) to choose one of a slice
        // let filter_options: [&str] = [];
        // let filter_options = ["Cool"];
        // let filter_options = ["Cool", "Hot", "Nice", "Close", "YaBoi"];

        let _scene_options2 = [
            Scene {
                id: 5,
                name: "BeginCam".to_string(),
            },
            // Scene {
            //     id: 4,
            //     name: "Screen".to_string(),
            // },
            Scene {
                id: 12,
                name: "twitchchat".to_string(),
            },
        ];
        // let scene_options = [5, 4, 12];
        // let choosen_scene =
        //     &scene_options2[rng().gen_range(0..scene_options2.len())];

        // println!("CHOOSEN SCENE: {:?}", choosen_scene);

        // let option = filter_options[rng().gen_range(0..filter_options.len())];
        // let filter_name = option;

        // let filter_details = obs_client
        //     .filters()
        //     .get(&choosen_scene.name.clone(), filter_name)
        //     .await?;
        // println!("Details {:?}", filter_details);
        // if DEBUG {
        //     println!("Details {:?}", filter_details);
        // }
        // // Enable Filter
        // let filter_enabled = obws::requests::filters::SetEnabled {
        //     source: &choosen_scene.name.clone(),
        //     filter: filter_name,
        //     enabled: !filter_details.enabled,
        // };
        // obs_client.filters().set_enabled(filter_enabled).await?;

        // pub const TEST_BROWSER: &str = "OBWS-TEST-Browser";
        // let settings = client
        //     .settings::<serde_json::Value>(TEST_BROWSER)
        //     .await?
        //     .settings;
        // client
        //     .set_settings(SetSettings {
        //         input: TEST_BROWSER,
        //         settings: &settings,
        //         overlay: Some(false),
        //     })
        //     .await?;

        // [SourceFilter { enabled: true, index: 0, kind: "chroma_key_filter_v2", name: "Chroma Key", settings: Object {"similarity": Number(431)} },
        // let filters = obs_client.filters().list("BeginCam").await?;
        // println!("Filters: {:?}", filters);

        // Enable Hot Filter
        // Enable blue Filter
        //

        // Then it's update MoveOpacity filter
        // Enable Filter

        // I just added a move-value filter on BeginCam called "MoveOpacity"
        // it moves the value of Opacity over 3 Seconds, when you trigger it
        //
        // if the Filter Hot is On
        // pub struct SetSettings<'a, T> {
        //     pub source: &'a str,
        //     pub filter: &'a str,
        //     pub settings: T,
        //     pub overlay: Option<bool>,
        // }

        //
        // Down Here let's update some Filters

        // ===================================================

        // This is the same as holding the Super key on an Ergodox
        let super_key = obws::requests::hotkeys::KeyModifiers {
            shift: true,
            control: true,
            alt: true,
            command: true,
        };

        let source = "BeginCam";
        let filter_name = "Move_SDF_Effects";

        // Outer Shadow Min/Max Distance, -16 -> 16
        //
        // Offset: 0 - 100
        //
        // Scale: 499
        // Threshold 100
        //
        // What is Color???????

        // Sharpness Range
        // let float_max = 100.0;

        // Alpha Range
        let float_min = 0.0;
        let float_max = 100.0;

        // Width Range
        // let float_min = 0.0;
        // let float_max = 16.0;

        match splitmsg[0].as_str() {
            "!outline" => {
                // move_sdf_effects
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: "BeginCam",
                    filter: "Move_SDF_Effects",
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }

            // All The Alphas
            // !outline Filter.SDFEffects.Glow.Outer.Alpha
            "!sdf" => {
                let filter_setting_name = splitmsg[1].as_str();
                let filter_details =
                    obs_client.filters().get(&source, &filter_name).await?;

                let mut new_settings =
                    serde_json::from_value::<MoveSingleValueSetting>(
                        filter_details.settings,
                    )
                    .unwrap();

                new_settings.setting_name = String::from(filter_setting_name);

                // We need a float max lookup
                new_settings.setting_float = float_max;

                let new_settings = obws::requests::filters::SetSettings {
                    source: &source,
                    filter: &filter_name,
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: &source,
                    filter: &filter_name,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }

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

            "!sub_outline_on" => {
                let filter_details =
                    obs_client.filters().get("BeginCam", "Outline").await?;

                println!("Filter Details: {:?}\n\n", filter_details);

                // thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an `Err`
                // value: Error("invalid type: floating point `0`, expected u32", line: 0, column:
                // 0)', src/bin/begin.rs:776:22

                let mut new_settings =
                    serde_json::from_value::<SDFEffectsSettings>(
                        filter_details.settings,
                    )
                    .unwrap();
                println!("\n\nOutline {:?}", new_settings);

                new_settings.glow_outer = Some(true);

                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "Outline",
                    settings: new_settings,
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

            "!uno" => {
                let filter_setting_name = splitmsg[1].as_str();
                let filter_details =
                    obs_client.filters().get(&source, &filter_name).await?;

                println!("{:?}", filter_details);
                let mut new_settings =
                    serde_json::from_value::<MoveSingleValueSetting>(
                        filter_details.settings,
                    )
                    .unwrap();

                new_settings.setting_name = String::from(filter_setting_name);

                new_settings.setting_float = float_min;

                let new_settings = obws::requests::filters::SetSettings {
                    source: &source,
                    filter: &filter_name,
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
            }
            "!do" => {
                let filter_setting_name = splitmsg[1].as_str();
                let filter_details =
                    obs_client.filters().get(&source, &filter_name).await?;

                println!("{:?}", filter_details);
                let mut new_settings =
                    serde_json::from_value::<MoveSingleValueSetting>(
                        filter_details.settings,
                    )
                    .unwrap();

                new_settings.setting_name = String::from(filter_setting_name);
                new_settings.setting_float = float_max;

                let new_settings = obws::requests::filters::SetSettings {
                    source: &source,
                    filter: &filter_name,
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;
            }
            "!update_outline" => {
                let filter_details =
                    obs_client.filters().get("BeginCam", "Outline").await?;
                let mut new_settings =
                    serde_json::from_value::<SDFEffectsSettings>(
                        filter_details.settings,
                    )
                    .unwrap();
                println!("\n\nOutline {:?}", new_settings);

                new_settings.glow_outer_width = Some(16.0);

                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "Outline",
                    settings: new_settings,
                    overlay: None,
                };
                obs_client.filters().set_settings(new_settings).await?;

                // This doesn't do anything????
                // So this is the call of it ???
                // .unwrap();
                // let new_settings: SDFEffectsSettings =
                //     serde_json::from_value(filter_details.settings).unwrap();

                // I can parse this into a Settings Object???
                // filter_details.settings.value;
                // let filter_details.settings.glow_outer_width = 0.0;

                // let new_filter = filters::Create {
                //     source: "BeginCam",
                //     filter: "TempFilter",
                //     kind: "streamfx-filter-sdf-effects",
                //     settings: Some(new_settings),
                // };
                // obs_client.filters().create(new_filter).await?;
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
                    camera_mode: 1,
                    commit: "g0f114f56".to_string(),
                    position_x: -0.009999999776482582,
                    position_y: float_amount,
                    // position_y: -30.0,
                    position_z: 0.019999999552965164,
                    rotation_x: 243.92999267578125,
                    rotation_y: -4.289999961853027,
                    rotation_z: -2.140000104904175,
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
            "!return" => {
                let settings: StreamFXSettings = StreamFXSettings {
                    camera_mode: 1,
                    commit: "g0f114f56".to_string(),
                    position_x: 0.0,
                    position_y: 0.0,
                    position_z: 0.0,
                    rotation_x: 0.0,
                    rotation_y: 0.0,
                    rotation_z: 0.0,
                    version: 51539607703,
                };
                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "YaBoi",
                    settings,
                    overlay: None,
                };
                obs_client
                    .filters()
                    .set_settings(new_settings)
                    .await
                    .unwrap();
            }
            "!fade" => {
                let opacity_settings = MoveOpacitySettings {
                    duration: 3000,
                    filter: "Hot".to_string(),
                    setting_float: 0.0,
                    setting_float_max: 1.0,
                    setting_float_min: 1.0,
                    setting_name: "opacity".to_string(),
                    value_type: 2,
                };
                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "MoveOpacity",
                    settings: opacity_settings,
                    overlay: None,
                };
                obs_client
                    .filters()
                    .set_settings(new_settings)
                    .await
                    .unwrap();
            }

            "!trigger" => {
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: "BeginCam",
                    filter: "MoveOpacity",
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }

            "!show" => {
                let opacity_settings = MoveOpacitySettings {
                    duration: 3000,
                    filter: "Hot".to_string(),
                    setting_float: 1.0,
                    setting_float_max: 1.0,
                    setting_float_min: 1.0,
                    setting_name: "opacity".to_string(),
                    value_type: 2,
                };
                let new_settings = obws::requests::filters::SetSettings {
                    source: "BeginCam",
                    filter: "MoveOpacity",
                    settings: opacity_settings,
                    overlay: None,
                };
                obs_client
                    .filters()
                    .set_settings(new_settings)
                    .await
                    .unwrap();
            }
            "!ya" => {
                let yaboi_details =
                    obs_client.filters().get("BeginCam", "YaBoi").await?;
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: "BeginCam",
                    filter: "YaBoi",
                    enabled: !yaboi_details.enabled,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }
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
            "!sbf" => {
                obs_client.scenes().set_current_program_scene("SBF").await?;
            }
            "!one" => {
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
