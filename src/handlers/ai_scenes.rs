use crate::ai_scene;
use crate::stream_character;
use crate::twitch_stream_state;
use ai_friends;
use ai_movie_trailers;
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
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use subd_audio;
use subd_types::AiScenesRequest;
use subd_types::Event;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

// Should this have an OBS Client as well?
pub struct AiScenesHandler {
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
        println!("Starting AI Scenes Handler");
        loop {
            let event = rx.recv().await?;
            let ai_scene_req = match event {
                Event::AiScenesRequest(msg) => msg,
                _ => continue,
            };

            // If this crashes we just want to loop again
            // and we expect the error to be printing
            let _ = ai_scenes_coordinator::run_ai_scene(
                &self.twitch_client,
                &self.obs_client,
                &self.pool,
                &self.elevenlabs,
                &ai_scene_req,
            )
            .await;
        }
    }
}

// =======================================================================================
