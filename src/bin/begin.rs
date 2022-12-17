use anyhow::{bail, Result};
use async_trait::async_trait;
use events::EventHandler;
use obws::requests::scene_items::Scale;
use obws::Client as OBSClient;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::*;
use rodio::{Decoder, OutputStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::{thread, time};
use subd_types::TransformOBSTextRequest;
use subd_types::TriggerHotkeyRequest;
use subd_types::UberDuckRequest;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

const DEFAULT_SCENE: &str = "Primary";

// We need a secondary scene, where we put all the jokes
const MEME_SCENE: &str = "memes";

// THE WORD DEFAULT IS DANGEROUS
const DEFAULT_SOURCE: &str = "begin";

// THESE NAMES AIN'T RIGHT!!!!
const DEFAULT_MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
const DEFAULT_MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";

pub struct TriggerHotkeyHandler {
    obs_client: OBSClient,
}

pub struct TransformOBSTextHandler {
    obs_client: OBSClient,
}

pub struct UberDuckHandler {
    sink: Sink,
}

pub struct SoundHandler {
    sink: Sink,
}

pub struct OBSMessageHandler {
    obs_client: OBSClient,
}

// This is really to show where the HotKeys are
#[derive(Debug)]
pub struct CharacterSetup {
    on: String,
    off: String,
    text_source: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckVoiceResponse {
    uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckFileResponse {
    path: Option<String>,
    started_at: Option<String>,
    failed_at: Option<String>,
    finished_at: Option<String>,
}

#[async_trait]
impl EventHandler for TriggerHotkeyHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TriggerHotkeyRequest(msg) => msg,
                _ => continue,
            };

            server::obs::trigger_hotkey(&msg.hotkey, &self.obs_client).await?;
        }
    }
}

#[async_trait]
impl EventHandler for TransformOBSTextHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TransformOBSTextRequest(msg) => msg,
                _ => continue,
            };

            println!(
                "Attempting to transform OBS Text: {:?} {:?}",
                &msg.text_source, &msg.message
            );
            server::obs::update_and_trigger_text_move_filter(
                &msg.text_source,
                "OBS_Text",
                &msg.message,
                &self.obs_client,
            )
            .await?;
        }
    }
}

#[async_trait]
impl EventHandler for UberDuckHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UberDuckRequest(msg) => msg,
                _ => continue,
            };

            let hotkey = find_obs_character(&msg.voice);
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: msg.message,
                    text_source: hotkey.text_source,
                },
            ));

            let (username, secret) = uberduck_creds();

            // This is every string???
            let client = reqwest::Client::new();
            let res = client
                .post("https://api.uberduck.ai/speak")
                .basic_auth(username.clone(), Some(secret.clone()))
                .json(&[("speech", msg.voice_text), ("voice", msg.voice)])
                .send()
                .await?
                .json::<UberDuckVoiceResponse>()
                .await?;

            let uuid = match res.uuid {
                Some(x) => x,
                None => continue,
            };

            // Start looping
            loop {
                let url = format!(
                    "https://api.uberduck.ai/speak-status?uuid={}",
                    &uuid
                );

                let (username, secret) = uberduck_creds();
                let response = client
                    .get(url)
                    .basic_auth(username, Some(secret))
                    .send()
                    .await?;

                let text = response.text().await?;
                println!("Uberduck Response: {:?}", text);
                let file_resp: UberDuckFileResponse =
                    serde_json::from_str(&text)?;

                // so now we need to match
                // Check if the failed_at parameter is null
                match file_resp.path {
                    Some(new_url) => {
                        // TODO Should we change this file name
                        let local_path = "./test.wav";
                        let response = client.get(new_url).send().await?;
                        let file = File::create(local_path)?;
                        let mut writer = BufWriter::new(file);
                        writer.write_all(&response.bytes().await?)?;
                        println!("Downloaded File From Uberduck, Playing Soon: {:?}!", local_path);

                        let file =
                            BufReader::new(File::open(local_path).unwrap());

                        // This is sending the request I thought!
                        let _ = tx.send(Event::TriggerHotkeyRequest(
                            TriggerHotkeyRequest { hotkey: hotkey.on },
                        ));

                        self.sink.append(
                            Decoder::new(BufReader::new(file)).unwrap(),
                        );
                        self.sink.sleep_until_end();

                        // THIS IS HIDING THE PERSON AFTER
                        // We might want to wait a little longer, then hide
                        // we could also kick off a hide event
                        // we wait 1 second, so they can read the text
                        let ten_millis = time::Duration::from_millis(1000);
                        thread::sleep(ten_millis);
                        let _ = tx.send(Event::TriggerHotkeyRequest(
                            TriggerHotkeyRequest { hotkey: hotkey.off },
                        ));
                        break;
                    }
                    None => {
                        // Wait 1 second before seeing if the file is ready.
                        let ten_millis = time::Duration::from_millis(1000);
                        thread::sleep(ten_millis);
                    }
                }
            }
        }
    }
}
#[async_trait]
impl EventHandler for SoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // TODO: Do I need to read there here either???
        let paths = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for path in paths {
            mp3s.insert(path.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;

            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // Can I put this out
            // We need to actually move this function
            // mickey-mouse
            // tommy-pickles
            // let default_voice = "goku".to_string();
            // let default_voice = "danny-devito-angry".to_string();
            let default_voice = "mojo-jojo".to_string();
            // let default_voice = "mojo-jojo".to_string();
            // let default_voice = "mickey-mouse".to_string();
            // let default_voice = "brock-samson".to_string();
            let voices: HashMap<String, String> = HashMap::from([
                ("beginbotbot".to_string(), "mr-krabs-joewhyte".to_string()),
                ("beginbot".to_string(), "danny-devito-angry".to_string()),
                // ("beginbotbot".to_string(), "theneedledrop".to_string()),
                // ("artmattdank".to_string(), "mojo-jojo".to_string()),
                ("ArtMattDank".to_string(), "dr-nick".to_string()),
                (
                    "carlvandergeest".to_string(),
                    "danny-devito-angry".to_string(),
                ),
                ("stupac62".to_string(), "stewie-griffin".to_string()),
                ("swenson".to_string(), "mike-wazowski".to_string()),
                ("teej_dv".to_string(), "mr-krabs-joewhyte".to_string()),
            ]);

            let voice = match voices.get(&msg.user_name) {
                Some(v) => v,
                None => &default_voice,
            };

            let mut seal_text = msg.contents.clone();
            let spaces: Vec<_> = msg.contents.match_indices(" ").collect();

            let line_length_modifier = 20;
            let mut line_length_limit = 20;

            for val in spaces.iter() {
                if val.0 > line_length_limit {
                    seal_text.replace_range(val.0..=val.0, "\n");
                    line_length_limit =
                        line_length_limit + line_length_modifier;
                }
            }
            let voice_text = msg.contents.to_string();

            // WE need ot manuplate!!!!
            // So it works here for some reason
            let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                message: seal_text,
                voice: voice.to_string(),
                voice_text,
            }));

            for word in splitmsg {
                let sanitized_word = word.as_str().to_lowercase();
                let full_name = format!("./MP3s/{}.mp3", sanitized_word);

                if mp3s.contains(&full_name) {
                    let file = BufReader::new(
                        File::open(format!("./MP3s/{}.mp3", sanitized_word))
                            .unwrap(),
                    );

                    self.sink
                        .append(Decoder::new(BufReader::new(file)).unwrap());

                    self.sink.sleep_until_end();
                }
            }
        }
    }
}

#[async_trait]
impl EventHandler for OBSMessageHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };
            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // we could send a tx

            // I could pass in the tx here
            // what is a beginmessage
            // why do we do this
            // we could handle other things here
            match handle_obs_commands(&tx, &self.obs_client, splitmsg, msg)
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}

// This is not ideal though
// I think we should try alternative filter triggering instead
// we need to trigger 3 filters each time
// and we can get the names based offa  pattern
// This is not the ideal method
fn find_obs_character(voice: &String) -> CharacterSetup {
    let mut hotkeys: HashMap<String, CharacterSetup> = HashMap::from([
        (
            "mr-krabs-joewhyte".to_string(),
            CharacterSetup {
                on: "OBS_KEY_0".to_string(),
                off: "OBS_KEY_1".to_string(),
                text_source: "mr.crabs-text".to_string(),
            },
        ),
        (
            "danny-devito-angry".to_string(),
            CharacterSetup {
                on: "OBS_KEY_2".to_string(),
                off: "OBS_KEY_3".to_string(),
                text_source: "Kevin-text".to_string(),
            },
        ),
    ]);

    let default_hotkeys = CharacterSetup {
        on: "OBS_KEY_6".to_string(),
        off: "OBS_KEY_7".to_string(),
        text_source: "Text".to_string(),
    };

    match hotkeys.remove(voice) {
        Some(v) => v,
        None => default_hotkeys,
    }
}

fn uberduck_creds() -> (String, String) {
    let username = env::var("UBER_DUCK_KEY")
        .expect("Failed to read UBER_DUCK_KEY environment variable");
    let secret = env::var("UBER_DUCK_SECRET")
        .expect("Failed to read UBER_DUCK_SECRET environment variable");
    (username, secret)
}

async fn handle_obs_commands(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
    msg: UserMessage,
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

        "!blur" => {
            let filter_value = splitmsg
                .get(2)
                .map_or(100.0, |x| x.trim().parse().unwrap_or(100.0));

            // msg.roles.is_twitch_mod()
            // msg.roles.is_twitch_founder()
            // msg.roles.is_twitch_staff()
            // msg.roles.is_twitch_sub()
            // if msg.roles.is_twitch_vip() {

            // So maybe the source is wrong
            // maybe the DEFAULT_MOVE_BLUR_FILTER_NAME name is wrong
            //
            // the 2 is also problematic
            // and we aren't pull in the duration
            server::obs::update_and_trigger_move_value_filter(
                source,
                DEFAULT_MOVE_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                filter_value,
                300,
                // 5000, // duration
                0,
                &obs_client,
            )
            .await?;

            // }

            Ok(())
        }

        // Update to take in 2 as a const
        "!noblur" | "!unblur" => {
            if msg.roles.is_twitch_mod() {
                println!("WE GOT A MOD OVER HERE");
                server::obs::update_and_trigger_move_value_filter(
                    source,
                    DEFAULT_MOVE_BLUR_FILTER_NAME,
                    "Filter.Blur.Size",
                    0.0,
                    5000,
                    2,
                    &obs_client,
                )
                .await?;
            }
            Ok(())
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

    // Create 1 Event Loop
    // Push handles onto the loop
    // those handlers are things like twitch-chat, twitch-sub, github-sponsor etc.
    let mut event_loop = events::EventLoop::new();

    // You can clone this
    // because it's just adding one more connection per clone()???
    //
    // This is useful because you need no lifetimes
    let pool = subd_db::get_db_pool().await;

    // Turns twitch IRC things into our message events
    event_loop.push(twitch_chat::TwitchChat::new(
        pool.clone(),
        "beginbot".to_string(),
    )?);

    // Does stuff with twitch messages
    event_loop.push(twitch_chat::TwitchMessageHandler::new(
        pool.clone(),
        twitch_service::Service::new(
            pool.clone(),
            user_service::Service::new(pool.clone()).await,
        )
        .await,
    ));
    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;
    event_loop.push(OBSMessageHandler { obs_client });

    // Works for Arch Linux
    let (_stream, stream_handle) = get_output_stream("pulse");

    // Works for Mac
    // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    event_loop.push(SoundHandler { sink });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    event_loop.push(UberDuckHandler { sink });

    // You need your own OBS client then
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;
    event_loop.push(TriggerHotkeyHandler { obs_client });

    // You need your own OBS client then
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;
    event_loop.push(TransformOBSTextHandler { obs_client });

    event_loop.run().await?;

    Ok(())
}
