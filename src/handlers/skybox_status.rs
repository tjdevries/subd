extern crate reqwest;
extern crate serde;
extern crate serde_json;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;
use tokio::time::sleep;

// static SKYBOX_STATUS_URL: &str =
//     "https://backend.blockadelabs.com/api/v1/imagine/requests";

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

// This should be moved into a handlers/ folder
#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxStatusHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            // do we wait for events???
            // let event = rx.recv().await?;
            // let request = match event {
            //     Event::SkyboxRequest(msg) => msg,
            //     _ => continue,
            // };

            // Check DB
            // Sleep 60 seconds
            sleep(Duration::from_secs(60)).await;
        }
    }
}
