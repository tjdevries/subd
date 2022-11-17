#![allow(dead_code)]

// TODO:
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound
//
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use std::fs;

use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;

use obws::requests::scene_items::SceneItemTransform;
use obws::requests::scene_items::SetTransform;
use obws::Client as OBSClient;

use server::commands;
use server::themesong;
use server::users;
use subd_types::Event;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

#[tracing::instrument(skip(_tx, rx))]
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
        // info!(sender = %msg.sender.name, badges = %badges, "{}", msg.message_text);

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
                let paths = fs::read_dir("./MP3s").unwrap();
                let example = splitmsg[0].as_str();
                let full_name = format!("./MP3s/{}.mp3", example);
                println!("{}", full_name);
                for path in paths {
                    if path.unwrap().path().display().to_string() == full_name {
                        println!("BIG!!! WIN");

                        // This works for Begin
                        let (_stream, stream_handle) =
                            get_output_stream("pulse");

                        // This works for Mac
                        // let (_stream, stream_handle) =
                        //     OutputStream::try_default().unwrap();
                        // Load a sound from a file, using a path relative to Cargo.toml
                        // let file =
                        //     BufReader::new(File::open(full_name2).unwrap());
                        let file = BufReader::new(
                            File::open(format!("./MP3s/{}.mp3", example))
                                .unwrap(),
                        );

                        let source = Decoder::new(file).unwrap();
                        stream_handle
                            .play_raw(source.convert_samples())
                            .expect("ok");
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    }
                }
            }
        };
    }
}

async fn handle_set_command<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    conn: &mut sqlx::SqliteConnection,
    client: &TwitchIRCClient<T, L>,
    msg: twitch_irc::message::PrivmsgMessage,
) -> Result<()> {
    let splitmsg = msg
        .message_text
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    // !set github <login>
    if splitmsg[1] == "github" {
        println!("  ... split msg: {:?}", splitmsg);

        if splitmsg.len() != 3 {
            say(client, format!("@{}: !set github <login>", msg.sender.name))
                .await?;
            return Ok(());
        }

        let user_id =
            subd_db::get_user_from_twitch_user(conn, &msg.sender.id).await?;

        let github_login = splitmsg[2].clone();
        subd_db::set_github_info_for_user(
            conn,
            &user_id,
            github_login.as_str(),
        )
        .await?;
        say(
            client,
            format!(
                "Succesfully set: twitch {} -> github {}",
                msg.sender.name, github_login
            ),
        )
        .await?;

        return Ok(());
    }

    // TODO(user_roles)
    if !msg
        .badges
        .iter()
        .any(|f| f.name == "broadcaster" || f.name == "moderator")
    {
        return Err(anyhow!("Not authorized User"));
    }

    if splitmsg[1] == "themesong" && splitmsg[2] == "unplayed" {
        let twitch_user = splitmsg[3].replace("@", "");
        let user_id =
            match subd_db::get_user_from_twitch_user_name(conn, &twitch_user)
                .await?
            {
                Some(user_id) => user_id,
                None => return Ok(()),
            };
        themesong::mark_themesong_unplayed(conn, &user_id).await?;
        println!(
            "  Successfully marked themesong unplayed for: {:?}",
            twitch_user
        );
    }

    Ok(())
}

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

#[tracing::instrument(skip(tx))]
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

    info!("waiting for messages");
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

// Here you wait for OBS Events, that are commands to trigger OBS
async fn handle_obs_stuff(
    _tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;

    // let version = obs_client.general().version().await?;
    // println!("OBS version: {:?}", version);

    let obs_test_scene = "Primary";
    obs_client
        .scenes()
        .set_current_program_scene(&obs_test_scene)
        .await?;

    println!("WAHT IS GOIUNG ON");
    let items = obs_client.scene_items().list("Primary").await?;
    println!("Items: {:?}", items);

    let details = obs_client.scene_items().transform("Primary", 43).await?;
    println!("Details {:?}", details);

    // Details SceneItemTransform {
    //   source_width: 500.0,
    //   source_height: 500.0,
    //   position_x: 492.0,
    //   position_y: 332.0,
    //   rotation: 33.0,
    //   scale_x: 1.0,
    //   scale_y: 1.0,
    //   width: 500.0,
    //   height: 500.0,
    //   alignment: LEFT | TOP,
    //   bounds_type: None,
    //   bounds_alignment: CENTER,
    //   bounds_width: 0.0,
    //   bounds_height: 0.0,
    //   crop_left: 0,
    //   crop_right: 0,
    //   crop_top: 0,
    //   crop_bottom: 0
    // }
    let new_rot = details.rotation + 10.0;

    // can we get the info???
    // So we we hitting the right Set Transform??
    let scene_transform = SceneItemTransform {
        rotation: Some(new_rot),
        alignment: None,
        bounds: None,
        crop: None,
        scale: None,
        position: None,
    };
    // HMMMM
    // so 43 is jonah
    // So We need to find the Item Numbers
    let set_transform = SetTransform {
        scene: "Primary",
        // item_id: 5, // BeginCam
        item_id: 43, // Jonah.gif
        transform: scene_transform,
    };

    // Is this working????
    obs_client
        .scene_items()
        .set_transform(set_transform)
        .await?;
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
        // info!(sender = %msg.sender.name, badges = %badges, "{}", msg.message_text);

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

        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Print all HotKeys
        // let result = obs_client.hotkeys().list().await?;
        // println!("HOT KEY TIME {result:?}");
        let keys = obws::requests::hotkeys::KeyModifiers {
            shift: true,
            control: true,
            alt: true,
            command: true,
        };

        let hotkey = splitmsg[0].as_str();
        println!("HotKey: {hotkey}");

        // let themesong = sqlx::query!(
        //     "SELECT song FROM user_theme_songs WHERE user_id = ?1",
        //     user_id
        // )
        // .fetch_optional(&mut *conn)
        // .await?;

        // What is song???

        // let themesong = match themesong {
        //     Some(themesong) => themesong,
        //     None => {
        //         println!("theme_song: No themesong available for: {:?}", user_id);
        //         return Ok(false);
        //     }
        // };

        // let rodioer =
        //     rodio::Decoder::new(BufReader::new(Cursor::new(themesong.song)))
        //         .unwrap();
        // // TODO: I would like to turn this off after the sink finishes playing, but I don't know how to
        // // do that yet, this probably wouldn't work with queued themesongs (for example)
        // // rodioer.total_duration();

        // sink.append(rodioer);

        // thread 'tokio-runtime-worker' panicked at 'this should work: API error: CannotAct', src/bin/begin.rs:369:5

        // we need conn, user_id and sink
        // themesong::play_themesong_for_today(&mut conn, &user_id, &sink).await?;
        // "key": "OBS_KEY_H",
        match splitmsg[0].as_str() {
            "!chat" => {
                obs_client
                    .hotkeys()
                    .trigger_by_sequence("OBS_KEY_L", keys)
                    .await?
            }

            "!code" => {
                obs_client
                    .hotkeys()
                    .trigger_by_sequence("OBS_KEY_H", keys)
                    .await?
            }
            "!hotkey" => {
                println!("HOT KEY TIME");
                // Doesn't Work
                // obs_client.hotkeys().trigger_by_name("MoveScreenH").await?
                obs_client
                    .hotkeys()
                    .trigger_by_name("libobs.hide_scene_item.BeginCam")
                    .await?
                // obs_client.hotkeys().trigger_by_sequence("L", keys).await?
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

    // If this because we fail here???
    // let stream_handle OutputStreamHandle = OutputStreamHandle{mixer: ""};
    // I don't want to try default!!!
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

#[tokio::main]
async fn main() -> Result<()> {
    // let devices = Devices {}.as_inner();
    // println!("DEVICES: {:?}", devices);
    // list_host_devices();

    // # Device : pulse
    // # Device : hdmi:CARD=HDMI,DEV=0
    // # Device : hdmi:CARD=HDMI,DEV=1
    // # Device : hdmi:CARD=HDMI,DEV=2
    // # Device : hdmi:CARD=HDMI,DEV=3
    // # Device : hdmi:CARD=HDMI,DEV=4
    // # Device : hdmi:CARD=HDMI,DEV=5
    // # Device : sysdefault:CARD=Generic
    // # Device : front:CARD=Generic,DEV=0
    // ALSA lib pcm_route.c:877:(find_matching_chmap) Found no matching channel map
    // # Device : surround21:CARD=Generic,DEV=0
    // # Device : surround40:CARD=Generic,DEV=0
    // ALSA lib pcm_route.c:877:(find_matching_chmap) Found no matching channel map
    // # Device : surround41:CARD=Generic,DEV=0
    // ALSA lib pcm_route.c:877:(find_matching_chmap) Found no matching channel map
    // # Device : surround50:CARD=Generic,DEV=0
    // # Device : surround51:CARD=Generic,DEV=0
    // # Device : surround71:CARD=Generic,DEV=0
    // # Device : iec958:CARD=Generic,DEV=0

    // Look if a file matched in a directory
    // let (_stream, stream_handle) = get_output_stream("pulse");
    // let (_stream, stream_handle) = get_output_stream("hdmi:CARD=HDMI,DEV=0");
    // let (_stream, stream_handle) = get_output_stream("sysdefault:CARD=Generic");
    // let sink = Sink::try_new(&stream_handle).unwrap();

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // Load a sound from a file, using a path relative to Cargo.toml
    // let file = BufReader::new(File::open("./MP3s/dontmakefun.mp3").unwrap());

    // let source = Decoder::new(file).unwrap();
    // stream_handle.play_raw(source.convert_samples());
    // std::thread::sleep(std::time::Duration::from_secs(5));

    // let file = BufReader::new(File::open("./MP3s/abc.mp3").unwrap());
    // let source = Decoder::new(file).unwrap();
    // stream_handle.play_raw(source.convert_samples());
    // std::thread::sleep(std::time::Duration::from_secs(5));

    // We need to strip off the beginning
    // let paths = fs::read_dir("./MP3s").unwrap();
    // let example = "abc";
    // let full_name = format!("./MP3s/{}.mp3", example);
    // println!("{}", full_name);
    // for path in paths {
    //     if path.unwrap().path().display().to_string() == full_name {
    //         println!("BIUG WIN")
    //     }
    // }

    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::new("chat=debug,server=debug"))
        .without_time()
        .with_target(false)
        .finish()
        .init();

    info!("Starting chat server");

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

        // Otherwise, run it like this
        (|$new_tx:ident, $new_rx:ident| $impl:block) => {{
            let ($new_tx, $new_rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move { $impl }));
        }};
    }

    makechan!(handle_twitch_chat);
    makechan!(handle_twitch_msg);
    makechan!(handle_obs_stuff);

    for c in channels {
        // Wait for all the channels to be done
        c.await?;
    }

    Ok(())
}
