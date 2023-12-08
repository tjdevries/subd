extern crate reqwest;
use std::io;
extern crate serde;
extern crate serde_json;
use crate::skybox;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;

// use chrono::prelude::*;
// use serde_json::json;
// use std::io;
// use std::io::prelude::*;

static SKYBOX_STATUS_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine/requests";

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratorData {
    pub prompt: String,
    pub negative_text: String,
    pub animation_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub context: Option<String>,
    pub prompt: String,
    pub caption_text: Option<String>,
    pub author_name: String,
    pub alias_id: Option<String>,
    pub alias_name: Option<String>,
    pub progress: i32,
    pub status: String,
    pub queue_position: i32,
    pub file_url: String,
    pub thumb_url: String,
    pub video_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub media_version: i32,
    pub public: i32,
    pub error_message: Option<String>,
    pub _type: String,
    pub generator_data: GeneratorData,
    pub count_favorites: i32,
    pub likes_count: i32,
    pub user_imaginarium_image_left: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterRequest {
    pub response: Response,
}

#[allow(dead_code)]
pub struct Skybox {
    pub name: String,
}

pub struct SkyboxStatusHandler {
    pub obs_client: OBSClient,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxStatusHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let request = match event {
                Event::SkyboxRequest(msg) => msg,
                _ => continue,
            };

            // We don't just need to call request here
            // this event isn't just pure request
            // we need to start somewhere else
            println!("Attempting to Skybox");
            // request_skybox(request.msg).await?;

            // Can I kick off another loop???

            // TODO: we will need to trigger the skybox OBS source
            // to refresh, after we get an updated Skybox
            // AND generate a fresh HTML page using pannellum
        }
    }
}
