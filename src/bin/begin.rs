use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::Decoder;
use rodio::*;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time;
use subd_db::get_db_pool;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use subd_types::UberDuckRequest;
use tokio::sync::broadcast;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub struct OBSMessageHandler {
    obs_client: OBSClient,
    pool: sqlx::PgPool,
}

pub struct TriggerHotkeyHandler {
    obs_client: OBSClient,
}

pub struct StreamCharacterHandler {
    obs_client: OBSClient,
}

pub struct SourceVisibilityHandler {
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

            let _ = server::obs_source::set_enabled(
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

            println!(
                "We are going to trigger a Stream Character: {}",
                msg.source
            );

            if msg.enabled {
                let _ = server::obs_combo::trigger_character_filters(
                    &msg.source,
                    &self.obs_client,
                    true,
                )
                .await;
            } else {
                let _ = server::obs_combo::trigger_character_filters(
                    &msg.source,
                    &self.obs_client,
                    false,
                )
                .await;
            }
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

            server::obs_hotkeys::trigger_hotkey(&msg.hotkey, &self.obs_client)
                .await?;
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
            server::move_transition::update_and_trigger_text_move_filter(
                &msg.text_source,
                &filter_name,
                &msg.message,
                &self.obs_client,
            )
            .await?;
        }
    }
}

// ================================================================================================

// Looks through raw-text
// to either
// play TTS
//
// or play soundeffects
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
                Event::UserMessage(msg) => msg,
                _ => continue,
            };
            if msg.user_name == "Nightbot" {
                continue;
            }

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
            let stream_character = server::uberduck::build_stream_character(
                &self.pool,
                &msg.user_name,
            )
            .await?;

            let split = voice_text.split(" ");
            let vec = split.collect::<Vec<&str>>();
            if vec.len() < 2 {
                continue;
            };

            let state =
                server::twitch_stream_state::get_twitch_state(&self.pool)
                    .await?;

            // I've got the state
            // This is where we need to look up state of the Twitch Stream:
            if msg.roles.is_twitch_staff() {
                println!("\n\tTwitch Staff's here");

                let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    // voice: "dr-robotnik-movie".to_string(),
                    voice: "half-life-scientist".to_string(),
                    message: seal_text,
                    voice_text,
                    username: msg.user_name,
                    source: Some("Randall".to_string()),
                }));
            } else if msg.roles.is_twitch_mod() {
                println!("We have a Mod!!!");

                let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    voice: "brock-samson".to_string(),
                    message: seal_text,
                    voice_text,
                    username: msg.user_name,
                    source: None,
                }));
            } else if msg.roles.is_twitch_sub() {
                let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    voice: stream_character.voice.clone(),
                    message: seal_text,
                    voice_text,
                    username: msg.user_name,
                    source: None,
                }));

            // If it's NOT sub-only
            } else if !state.sub_only_tts {
                let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    voice: stream_character.voice.clone(),
                    message: seal_text,
                    voice_text,
                    username: msg.user_name,
                    source: None,
                }));
            }

            if !state.implicit_soundeffects {
                continue;
            }

            // IF EXplict continue
            // =============================
            // THIS IS JUST SILENCING SOUNDS
            // =============================

            // We should add this to a DB lookup
            // continue;
            let text_source = "Soundboard-Text";

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // This also needs the OTHER WORD EFFECT!!!!
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

                    // self.sink.volume()
                    // self.sink.set_volume()
                    // self.sink.len()

                    // We need this so we can trigger the next word
                    // Not sure we need this
                    let ten_millis = time::Duration::from_millis(100);
                    thread::sleep(ten_millis);
                }
            }

            // This might be right
            // So this is triggering and going to fast to the next
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source: text_source.to_string(),
                },
            ));
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

            match server::obs_routing::handle_obs_commands(
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

    // codyphobe:
    //
    // For the OBSClient cloning,
    // could you pass the OBSClient in the constructor when making event_loop,
    // then pass self.obsclient into each handler's handle method inside
    // EventLoop#run

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
    let obs_client = OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await?;

    event_loop.push(OBSMessageHandler {
        obs_client,
        pool: pool.clone(),
    });

    // Works for Arch Linux
    let (_stream, stream_handle) = server::audio::get_output_stream("pulse");
    // Works for Mac
    // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    event_loop.push(SoundHandler {
        sink,
        pool: pool.clone(),
    });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let pool = get_db_pool().await;
    event_loop.push(server::uberduck::UberDuckHandler { pool, sink });

    let obs_client = OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await?;
    event_loop.push(TriggerHotkeyHandler { obs_client });

    let obs_client = OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await?;
    event_loop.push(TransformOBSTextHandler { obs_client });

    let obs_client = OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await?;
    event_loop.push(StreamCharacterHandler { obs_client });

    let obs_client = OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await?;
    event_loop.push(SourceVisibilityHandler { obs_client });

    println!("\n\n\t\tLet's Start this Loop Up!");
    event_loop.run().await?;

    Ok(())
}
