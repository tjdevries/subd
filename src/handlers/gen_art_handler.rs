use crate::ai_scene;
use crate::audio;
use crate::openai::dalle;
use crate::redirect;
use crate::stream_character;
use crate::twitch_stream_state;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use events::EventHandler;
use obws::Client as OBSClient;
use rand::{seq::SliceRandom, thread_rng};
use rodio::*;
use stable_diffusion::models::GenerateAndArchiveRequest;
use stable_diffusion::models::RequestType;
use stable_diffusion::run_from_prompt;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::sync::Arc;
use subd_types::AiScenesRequest;
use subd_types::Event;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use twitch_chat::client::send_message;

use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

// ============================================================
//
// Should this have an OBS Client as well
pub struct GenArtHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pub elevenlabs: Elevenlabs,
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for AiScenesHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let twitch_client = Arc::new(Mutex::new(self.twitch_client));
        let clone_twitch_client = twitch_client.clone();
        let locked_client = clone_twitch_client.lock().await;

        loop {
            let event = rx.recv().await?;
            let ai_scene_req = match event {
                Event::GenArtRequest(msg) => msg,
                _ => continue,
            };
            
            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match handle_gen_art_requests(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
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

pub async fn handle_gen_art_requests(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let default_source = constants::DEFAULT_SOURCE.to_string();
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let is_mod = msg.roles.is_twitch_mod();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let _duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
    let _scene = obs_scenes::find_scene(source)
        .await
        .unwrap_or(constants::MEME_SCENE.to_string());
    let command = splitmsg[0].as_str();

    let _ = match command {
        "!art" => {
            println!("ART TIME");
            Ok(())
        }

        _ => Ok(()),
    };

    Ok(())
}

