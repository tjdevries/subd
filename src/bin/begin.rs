use anyhow::Result;
use async_trait::async_trait;
use csv::Writer;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::Decoder;
use rodio::*;
use serde::{Deserialize, Serialize};
use server::audio;
use server::move_transition;
use server::obs_combo;
use server::obs_hotkeys;
use server::obs_routing;
use server::obs_source;
use server::twitch_stream_state;
use server::uberduck;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time;
use subd_db::get_db_pool;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;
use tracing_subscriber;
// use tracing_subscriber::util::SubscriberInitExt;
// use tracing_subscriber::EnvFilter;

#[allow(dead_code)]
pub struct Skybox {
    pool: sqlx::PgPool,
    name: String,
}

#[allow(dead_code)]
pub struct SkyboxHandler {
    pool: sqlx::PgPool,
}

#[allow(dead_code)]
pub struct SkyboxRemixHandler {
    pool: sqlx::PgPool,
}

pub struct OBSMessageHandler {
    obs_client: OBSClient,
    pool: sqlx::PgPool,
}

pub struct TriggerHotkeyHandler {
    obs_client: OBSClient,
}

pub struct SourceVisibilityHandler {
    obs_client: OBSClient,
}

pub struct StreamCharacterHandler {
    obs_client: OBSClient,
}

pub struct TransformOBSTextHandler {
    obs_client: OBSClient,
}

pub struct SoundHandler {
    sink: Sink,
    pool: sqlx::PgPool,
}

// ================================================================================================

#[async_trait]
impl EventHandler for SourceVisibilityHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::SourceVisibilityRequest(msg) => msg,
                _ => continue,
            };

            let _ = obs_source::set_enabled(
                &msg.scene,
                &msg.source,
                msg.enabled,
                &self.obs_client,
            )
            .await;
        }
    }
}

#[async_trait]
impl EventHandler for StreamCharacterHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::StreamCharacterRequest(msg) => msg,
                _ => continue,
            };

            let _ = obs_combo::trigger_character_filters(
                &msg.source,
                &self.obs_client,
                msg.enabled,
            )
            .await;
        }
    }
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

            obs_hotkeys::trigger_hotkey(&msg.hotkey, &self.obs_client).await?;
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

            let filter_name = format!("Transform{}", msg.text_source);
            let _ = move_transition::update_and_trigger_text_move_filter(
                &msg.text_source,
                &filter_name,
                &msg.message,
                &self.obs_client,
            )
            .await;
        }
    }
}

// ================================================================================================

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Character {
    pub voice: Option<String>,
    pub source: Option<String>,
}

// Looks through raw-text to either play TTS or play soundeffects
#[async_trait]
impl EventHandler for SoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let paths = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for path in paths {
            mp3s.insert(path.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => {
                    // TODO: Add a list here
                    if msg.user_name == "Nightbot" {
                        continue;
                    }
                    msg
                }
                _ => continue,
            };

            // if msg.roles.is_twitch_staff() {

            let spoken_string = msg.contents.clone();
            let voice_text = msg.contents.to_string();
            let _speech_bubble_text = uberduck::chop_text(spoken_string);

            // Anything less than 3 words we don't use
            let split = voice_text.split(" ");
            let vec = split.collect::<Vec<&str>>();
            if vec.len() < 2 {
                continue;
            };

            // What does this do?
            let stream_character =
                uberduck::build_stream_character(&self.pool, &msg.user_name)
                    .await?;

            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;

            let voice = stream_character.voice.clone();

            // voice: stream_character.voice,
            let mut character = Character {
                voice: Some(voice),
                ..Default::default()
            };

            // See if I'm none of these!!!!
            //
            // This is all about how to respond to messages from various
            // types of users
            if msg.roles.is_twitch_staff() {
                character.voice =
                    Some(server::obs::TWITCH_STAFF_OBS_SOURCE.to_string());
                character.source =
                    Some(server::obs::TWITCH_STAFF_VOICE.to_string());
            } else if msg.user_name == "beginbotbot" {
                // TODO: Get better voice
                character.voice =
                    Some(server::obs::TWITCH_HELPER_VOICE.to_string());
                // character.voice = Some("stephen-a-smith".to_string());
                // Some("stephen-a-smith".to_string())
            } else if msg.roles.is_twitch_mod() {
                // character.voice =
                //     Some(server::obs::TWITCH_MOD_DEFAULT_VOICE.to_string());
            } else if msg.roles.is_twitch_sub() {
                character.voice = Some(stream_character.voice.clone());
            } else if !state.sub_only_tts {
                // This is what everyone get's to speak with
                // if we are allowing non-subs to speak
                character.voice = Some(stream_character.voice.clone());
            }

            // If the character
            // If we have a voice assigned, then we fire off an UberDuck Request
            match character.voice {
                Some(voice) => {
                    let records = vec![Record {
                        field_1: voice.clone(),
                        field_2: voice_text.clone(),
                    }];

                    // Write records to a CSV file
                    let csv_path =
                        "/home/begin/code/BeginGPT/tmp/voice_character.csv";
                    write_records_to_csv(&csv_path, &records)?;

                    // let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    //     voice,
                    //     message: speech_bubble_text,
                    //     voice_text,
                    //     username: msg.user_name,
                    //     source: character.source,
                    // }));
                }
                None => {}
            }

            // If we have the implicit_soundeffects enabled
            // we go past this!
            if state.implicit_soundeffects {
                continue;
            }

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let text_source =
                server::obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string();
            for word in splitmsg {
                let sanitized_word = word.as_str().to_lowercase();
                let full_name = format!("./MP3s/{}.mp3", sanitized_word);

                if mp3s.contains(&full_name) {
                    let _ = tx.send(Event::TransformOBSTextRequest(
                        TransformOBSTextRequest {
                            message: sanitized_word.clone(),
                            text_source: text_source.to_string(),
                        },
                    ));

                    let file = BufReader::new(
                        File::open(format!("./MP3s/{}.mp3", sanitized_word))
                            .unwrap(),
                    );

                    self.sink
                        .append(Decoder::new(BufReader::new(file)).unwrap());

                    self.sink.sleep_until_end();

                    // TODO: Look into using these!
                    // self.sink.volume()
                    // self.sink.set_volume()
                    // self.sink.len()

                    // We need this so we can allow to trigger the next word in OBS
                    // TODO: We should abstract
                    // and figure out a better way of determine the time
                    let sleep_time = time::Duration::from_millis(100);
                    thread::sleep(sleep_time);
                }
            }

            // This clears the OBS Text
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source: text_source.to_string(),
                },
            ));
        }
    }
}

// Define a custom data structure to hold the values
#[derive(Serialize)]
struct Record {
    field_1: String,
    field_2: String,
}

fn write_records_to_csv(path: &str, records: &[Record]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
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

            // THEORY: We don't know if this is an explicit OBS message at this stage
            println!("\n{:?}", msg);
            match obs_routing::handle_obs_commands(
                &tx,
                &self.obs_client,
                &self.pool,
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

// ==== //
// Main //
// ==== //

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        // .with_env_filter(EnvFilter::new("chat=debug,server=debug"))
        .without_time()
        .with_target(false)
        // .finish()
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

    // Advice!
    // codyphobe:
    //           For the OBSClient cloning,
    //           could you pass the OBSClient in the constructor when making event_loop,
    //           then pass self.obsclient into each handler's handle method inside
    //           EventLoop#run

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

    // This really is named wrong
    // this handles more than OBS
    // and it's also earlier in the program
    // but it takes an obs_client and pool none-the-less
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(OBSMessageHandler {
        obs_client,
        pool: pool.clone(),
    });

    // TODO: This should be abstracted
    // Works for Arch Linux
    let (_stream, stream_handle) =
        audio::get_output_stream("pulse").expect("stream handle");
    // Works for Mac
    // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    event_loop.push(SoundHandler {
        sink,
        pool: pool.clone(),
    });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let pool = get_db_pool().await;

    // Uberduck handles voice messages
    event_loop.push(uberduck::UberDuckHandler { pool, sink });

    // // OBS Hotkeys are controlled here
    // let obs_client = server::obs::create_obs_client().await?;
    // event_loop.push(TriggerHotkeyHandler { obs_client });
    //
    // // OBS Text is controlled here
    // let obs_client = server::obs::create_obs_client().await?;
    // event_loop.push(TransformOBSTextHandler { obs_client });
    //
    // // OBS Sources are controlled here
    // let obs_client = server::obs::create_obs_client().await?;
    // event_loop.push(SourceVisibilityHandler { obs_client });
    //
    // // OBS Stream Characters are controlled here
    // let obs_client = server::obs::create_obs_client().await?;
    // event_loop.push(StreamCharacterHandler { obs_client });

    // let _ = main().await;
    event_loop.run().await?;
    println!("\n\n\t\tStarting begin.rs!");
    println!("====================================================\n\n");

    // Shouldn't this pause?
    // So we are failing at the ok????
    Ok(())
}
