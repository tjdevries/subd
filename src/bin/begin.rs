use anyhow::Result;
use async_trait::async_trait;
// use csv::Writer;
use events::EventHandler;
use obws::Client as OBSClient;
// use serde::{Deserialize, Serialize};
use server::audio;
use server::move_transition;
use server::obs_combo;
use server::obs_hotkeys;
use server::obs_source;
// use server::twitch_stream_state;
use server::uberduck;
use server::handlers;
// use std::collections::HashSet;
use subd_db::get_db_pool;
use subd_types::Event;
// use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;

// use tracing_subscriber;

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
                _ => continue, }; let _ = obs_source::set_enabled(
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

// ============ //
// === Main === //
// ============ //

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: The tracing subscriber API updated
    // not sure how to set max level
    //
    // I think this is what is spitting out the Error Message
    // tracing_subscriber::fmt()
    //     // .with_max_level(Level::TRACE)
    //     // .with_env_filter(EnvFilter::new("chat=debug,server=debug"))
    //     .without_time()
    //     .with_target(false)
    //     // .finish()
    //     .init();

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

    // TODO: Update this description to be more exact
    // Saves the message and extracts out some information
    // for easier routing
    event_loop.push(twitch_chat::TwitchMessageHandler::new(
        pool.clone(),
        twitch_service::Service::new(
            pool.clone(),
            user_service::Service::new(pool.clone()).await,
        )
        .await,
    ));
    //
    // This really is named wrong
    // this handles more than OBS
    // and it's also earlier in the program
    // but it takes an obs_client and pool none-the-less
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(handlers::obs_messages::OBSMessageHandler {
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

    event_loop.push(handlers::sound_handler::SoundHandler {
        sink,
        pool: pool.clone(),
    });

    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let pool = get_db_pool().await;

    // Uberduck handles voice messages
    event_loop.push(uberduck::UberDuckHandler { pool, sink });

    // // OBS Hotkeys are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(TriggerHotkeyHandler { obs_client });
    //
    // // OBS Text is controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(TransformOBSTextHandler { obs_client });
    //
    // // OBS Sources are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(SourceVisibilityHandler { obs_client });
    //
    // // OBS Stream Characters are controlled here
    let obs_client = server::obs::create_obs_client().await?;
    event_loop.push(StreamCharacterHandler { obs_client });

    // let _ = main().await;
    event_loop.run().await?;
    println!("\n\n\t\tStarting begin.rs!");
    println!("====================================================\n\n");

    Ok(())
}
