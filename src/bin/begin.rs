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

pub struct TriggerHotkeyHandler {
    obs_client: OBSClient,
}

pub struct TransformOBSTextHandler {
    obs_client: OBSClient,
}

pub struct SoundHandler {
    sink: Sink,
}

pub struct OBSMessageHandler {
    obs_client: OBSClient,
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
            match server::obs::handle_obs_commands(
                &tx,
                &self.obs_client,
                splitmsg,
                msg,
            )
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
    event_loop.push(server::uberduck::UberDuckHandler { sink });

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
