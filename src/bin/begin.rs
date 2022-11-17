#![allow(dead_code)]

use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use std::fs;

use rand::Rng;
use serde::Serialize;

use rand::thread_rng as rng;

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

// use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

use obws::requests::scene_items::SceneItemTransform;
use obws::requests::scene_items::SetTransform;
use obws::Client as OBSClient;

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

        let twitch_username =
            subd_types::consts::get_twitch_broadcaster_username();
        match splitmsg[0].as_str() {
            "!echo" => {
                let echo = commands::Echo::try_parse_from(&splitmsg);
                if let Ok(echo) = echo {
                    let _ = client.say(twitch_username, echo.contents).await;
                }
            }
            _ => {
                // This handles all the Sound playing so Far
                // Maybe we could abstract this to another function
                let paths = fs::read_dir("./MP3s").unwrap();
                let example = splitmsg[0].as_str();
                let full_name = format!("./MP3s/{}.mp3", example);
                for path in paths {
                    if path.unwrap().path().display().to_string() == full_name {
                        // This works for Begin's Arch computer
                        let (_stream, stream_handle) =
                            get_output_stream("pulse");

                        // This works for Mac
                        // let (_stream, stream_handle) =
                        //     OutputStream::try_default().unwrap();

                        let file = BufReader::new(
                            File::open(format!("./MP3s/{}.mp3", example))
                                .unwrap(),
                        );

                        let source = Decoder::new(file).unwrap();

                        // Is this outputing the ALSA message????
                        stream_handle
                            .play_raw(source.convert_samples())
                            .expect("ok");

                        // There's a better a way of doing this
                        std::thread::sleep(std::time::Duration::from_secs(10));
                    }
                }
            }
        };
    }
}

// TODO: probably handle errors here
async fn change_scene(
    obs_client: &obws::client::Client,
    name: &str,
) -> Result<()> {
    obs_client.scenes().set_current_program_scene(&name).await?;
    Ok(())
}

// ==============================================================================

#[derive(Debug)]
pub struct Scene {
    id: i64,
    name: String,
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

    let obs_test_scene = "Primary";
    change_scene(&obs_client, &obs_test_scene).await?;

    let items = obs_client.scene_items().list(obs_test_scene).await?;
    if DEBUG {
        println!("Items: {:?}", items);
    }

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        // Enable Filter
        let filter_enabled = obws::requests::filters::SetEnabled {
            source: "BeginCam",
            filter: "Hot",
            enabled: true,
        };
        obs_client.filters().set_enabled(filter_enabled).await?;

        // Flip filters
        // Switch to Scenes
        // TODO: Update Filters

        // let filter_name = "WHA";

        // cafce25: if you use rand::seq::SliceRandom; you can options.choose(rng()) to choose one of a slice
        // let filter_options: [&str] = [];
        // let filter_options = ["Cool"];
        // let filter_options = ["Cool", "Hot", "Nice", "Close", "YaBoi"];

        let scene_options2 = [
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
        let choosen_scene =
            &scene_options2[rng().gen_range(0..scene_options2.len())];

        println!("CHOOSEN SCENE: {:?}", choosen_scene);

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

        let details = obs_client
            .scene_items()
            .transform(obs_test_scene, choosen_scene.id)
            .await?;
        if DEBUG {
            println!("Details {:?}", details);
        }

        // cafce25: no gen_range will return a value of the range
        //
        // TODO: Move this out!!!
        // Update a Scene's Settings
        // let rand_rot = rng.gen_range(0..100) as f32;
        // e.g. `thread_rng().gen::<i32>()`, or cached locally, e.g.
        let new_rot = details.rotation + (rng().gen_range(0..10) as f32);
        // rng.gen::<f32>();
        // let new_rot = details.rotation + 2.0;

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

        // Split Message
        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // This is the same as holding the Super key on an Ergodox
        let super_key = obws::requests::hotkeys::KeyModifiers {
            shift: true,
            control: true,
            alt: true,
            command: true,
        };

        match splitmsg[0].as_str() {
            "!rand" => {
                let amount = splitmsg[1].as_str();

                // how do I handle that
                let float_amount = match amount.parse::<f32>() {
                    Ok(val) => val,
                    Err(_) => {
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
