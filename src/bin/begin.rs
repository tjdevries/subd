#![allow(dead_code)]

use anyhow::{bail, Result};
use clap::Parser;
use obws::requests::scene_items::Scale;
use obws::Client as OBSClient;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use rodio::{Decoder, OutputStream};
use server::commands;
use server::users;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
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

const DEFAULT_SCENE: &str = "Primary";

// We need a secondary scene, where we put all the jokes
const MEME_SCENE: &str = "memes";

// THE WORD DEFAULT IS DANGEROUS
const DEFAULT_SOURCE: &str = "begin";

// THESE NAMES AIN'T RIGHT!!!!
const DEFAULT_MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
const DEFAULT_MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";

// Here you wait for OBS Events that are commands to trigger OBS
async fn handle_obs_stuff(
    _tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    // Connect to OBS
    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;

    // Connect to Twitch
    // let config = get_chat_config();
    //
    // This is used for !filter
    // Which we aren't currently using
    // let (_, client) = TwitchIRCClient::<
    //     SecureTCPTransport,
    //     StaticLoginCredentials,
    // >::new(config);
    // // used for !filter
    // let twitch_username = subd_types::consts::get_twitch_bot_username();

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        match handle_obs_commands(&obs_client, splitmsg).await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {err}");
                continue;
            }
        }
    }
}

async fn handle_obs_commands(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    // This is because Begin doesn't understand Rust
    let default_source = String::from(DEFAULT_SOURCE);

    // We try and do some parsing on every command here
    // These may not always be what we want, but they are sensible
    // defaults used by many commands
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));

    // WE PANICKED!!!!!!!
    let filter_value = splitmsg
        .get(3)
        .map_or(0.0, |x| x.trim().parse().unwrap_or(0.0));

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    // we need to figure a way to look up the defaults per command
    // because they could be different types
    // for now we are going to try and have them be the same
    // let filter_setting_name = splitmsg.get(2).map_or("", |x| x.as_str());

    match splitmsg[0].as_str() {
        // ================== //
        // Scrolling Sources //
        // ================== //

        // !scroll SOURCE SCROLL_SETTING SPEED DURATION (in milliseconds)
        // !scroll begin x 5 300
        //
        // TODO: Stop using server::obs::handle_user_input
        "!scroll" => {
            let default_filter_setting_name = String::from("speed_x");

            // This is ok, because we have a different default
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);

            let filter_setting_name: String = match filter_setting_name.as_str()
            {
                "x" => String::from("speed_x"),
                "y" => String::from("speed_y"),
                _ => default_filter_setting_name,
            };

            // TODO: THIS 2 is SUPERFLUOUS!!!
            // WE SHOULD RE-WRITE THIS METHOD NOT TO USE IT
            server::obs::handle_user_input(
                source,
                DEFAULT_MOVE_SCROLL_FILTER_NAME,
                &filter_setting_name,
                filter_value,
                duration,
                2,
                &obs_client,
            )
            .await
        }

        // We could maybe get this into one function
        // and have the word blur actually there
        // =============== //
        // Bluring Sources //
        // =============== //
        "!blur" => {
            let blur_size: f32 = splitmsg
                .get(2)
                .and_then(|x| x.trim().parse().ok())
                .unwrap_or(100.0);

            // THESE DON'T NEED THE GODDAMN 2!!!
            // TODO: we should print off error here
            server::obs::update_and_trigger_move_value_filter(
                source,
                DEFAULT_MOVE_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                blur_size,
                5000,
                2,
                &obs_client,
            )
            .await
        }

        // Update to take in 2 as a const
        "!noblur" | "!unblur" => {
            server::obs::update_and_trigger_move_value_filter(
                source,
                DEFAULT_MOVE_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                0.0,
                5000,
                2,
                &obs_client,
            )
            .await
        }

        // =============== //
        // Scaling Sources //
        // =============== //
        "!grow" | "!scale" => {
            let x: f32 = splitmsg
                .get(2)
                .and_then(|temp_x| temp_x.trim().parse().ok())
                .unwrap_or(1.0);
            let y: f32 = splitmsg
                .get(3)
                .and_then(|temp_y| temp_y.trim().parse().ok())
                .unwrap_or(1.0);

            let base_scale = Scale {
                x: Some(x),
                y: Some(y),
            };
            server::obs::trigger_grow(source, &base_scale, x, y, &obs_client)
                .await
        }

        // ====================== //
        // 3D Transforming Sources//
        // ====================== //

        // This shit is annoying
        // I almost want to divide it into 3 commands
        // based on Camera Type
        // and we have all 3
        // that might be too much
        // but i also might be exactly what we want
        // only spin is wonky
        // Should also add !spinz
        "!spin" | "!spinx" | "spiny" => {
            // HMMMMM
            let default_filter_setting_name = String::from("z");
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);

            server::obs::spin(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!hide" => server::obs::hide_sources(MEME_SCENE, &obs_client).await,
        "!show" => {
            server::obs::set_enabled(MEME_SCENE, source, true, &obs_client)
                .await
        }
        "!def_ortho" => {
            server::obs::default_ortho(source, duration, &obs_client).await
        }
        "!ortho" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            let filter_setting_name = &splitmsg[2];

            server::obs::trigger_ortho(
                source,
                "3D_Orthographic",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!perp" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            let filter_setting_name = &splitmsg[2];

            server::obs::trigger_ortho(
                source,
                "3D_Perspective",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!corner" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            let filter_setting_name = &splitmsg[2];

            server::obs::trigger_ortho(
                source,
                "3D_CornerPin",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }
        // Perspective
        // Corner Pin
        // Orthographic

        // !3d SOURCE FILTER_NAME FILTER_VALUE DURATION
        // !3d begin Rotation.Z 3600 5000
        //
        // TODO: This is NOT Working!
        "!3d" => {
            // If we don't at least have a filter_name, we can't proceed
            if splitmsg.len() < 3 {
                bail!("We don't have a filter name, can't proceed");
            }

            let filter_setting_name = &splitmsg[2];

            server::obs::trigger_3d(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        // ============== //
        // Moving Sources //
        // ============== //
        "!move" => {
            // TODO: Look at this fanciness
            //       cafce25: if let [source, x, y, ..] = splitmsg {...}
            if splitmsg.len() > 3 {
                let source = splitmsg[1].as_str();
                let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
                let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);

                server::obs::move_source(source, x, y, &obs_client).await
            } else {
                Ok(())
            }
        }

        // TODO: I'd like one-for every corner
        "!tr" => server::obs::top_right(source, &obs_client).await,

        "!bl" => server::obs::bottom_right(source, &obs_client).await,

        // ================ //
        // Compound Effects //
        // ================ //
        "!norm" => server::obs::norm(&source, &obs_client).await,

        "!follow" => {
            let scene = DEFAULT_SCENE;
            let leader = splitmsg.get(1).unwrap_or(&default_source);
            let source = leader;

            server::obs::follow(source, scene, leader, &obs_client).await
        }
        "!staff" => server::obs::staff(DEFAULT_SOURCE, &obs_client).await,

        // =============================== //
        // Create Scenes, Sources, Filters //
        // =============================== //
        "!create_source" => {
            let new_scene: obws::requests::scene_items::CreateSceneItem =
                obws::requests::scene_items::CreateSceneItem {
                    scene: DEFAULT_SCENE,
                    source: &source,
                    enabled: Some(true),
                };

            // TODO: Why is this crashing???
            obs_client.scene_items().create(new_scene).await?;

            Ok(())
        }

        // TEMP: This is for temporary testing!!!!
        "!split" => {
            server::obs::create_split_3d_transform_filters(source, &obs_client)
                .await
        }

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            server::obs::create_filters_for_source(source, &obs_client).await
        }

        // ========================== //
        // Show Info About OBS Setup  //
        // ========================== //
        // "!filter" => {
        //     let (_command, words) = msg.message_text.split_once(" ").unwrap();

        //     // TODO: Handle this error
        //     let details =
        //         server::obs::print_filter_info(&source, words, &obs_client)
        //             .await?;
        //     client
        //         .say(twitch_username.clone(), format!("{:?}", details))
        //         .await
        // }

        // TODO: Take in Scene
        "!source" => {
            server::obs::print_source_info(source, DEFAULT_SCENE, &obs_client)
                .await
        }

        "!outline" => {
            let source = splitmsg[1].as_str();
            server::obs::outline(source, &obs_client).await
        }

        // ====================== //
        // Show / Hide Subscenes //
        // ====================== //
        "!memes" => {
            server::obs::set_enabled(
                DEFAULT_SCENE,
                MEME_SCENE,
                true,
                &obs_client,
            )
            .await
        }

        "!nomemes" | "!nojokes" | "!work" => {
            server::obs::set_enabled(
                DEFAULT_SCENE,
                MEME_SCENE,
                false,
                &obs_client,
            )
            .await
        }

        // ==================== //
        // Change Scenes in OBS //
        // ==================== //
        // Rename These Commands
        "!chat" => server::obs::trigger_hotkey("OBS_KEY_L", &obs_client).await,

        "!code" => server::obs::trigger_hotkey("OBS_KEY_H", &obs_client).await,

        _ => Ok(()),
    }
}

// ==============================================================================
// ==============================================================================
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

// ==========================================================================================

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

// ===================================================================================================

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

        let badges = msg
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
        let user_roles =
            users::update_user_roles_once_per_day(&mut conn, &user_id, &msg)
                .await?;

        println!("User {:?} | {:?} | {:?}", msg.sender, user_roles, badges);

        for role in user_roles.roles {
            println!("{:?}", role);
        }

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
                // TODO: find an easy way to not start this code with a flag
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
                            Decoder::new(BufReader::new(file)).unwrap(),
                        );

                        sink.sleep_until_end();
                    }
                }
            }
        };
    }
}

