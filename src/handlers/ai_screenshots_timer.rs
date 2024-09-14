use crate::openai::dalle;
use crate::telephone;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use rand::seq::SliceRandom;
use rodio::*;
use std::thread;
use std::time;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AiScreenshotsTimerHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for AiScreenshotsTimerHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            match handle_ai_screenshots(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }

            // let t = time::Duration::from_millis(3000);
            let t = time::Duration::from_millis(1000);
            thread::sleep(t);
        }
    }
}

pub async fn handle_ai_screenshots(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    sink: &Sink,
) -> Result<String> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename =
        format!("../../tmp/screenshots/timelapse/{}", unique_identifier);

    let random_prompt = generate_random_prompt();
    let req = dalle::DalleRequest {
        prompt: random_prompt.clone(),
        username: "beginbot".to_string(),
        amount: 1,
    };
    telephone::create_screenshot_variation(
        sink,
        obs_client,
        filename,
        telephone::ImageRequestType::Dalle(req),
        random_prompt,
        "begin".to_string(),
        Some("timelapse".to_string()),
    )
    .await
}

// This is key
pub fn generate_random_prompt() -> String {
    let choices = vec![
        "an 80's anime".to_string(),
        // "as a Pepe the frog".to_string(),
        // "album cover".to_string(),
        "as a ripped dude".to_string(),
        "as a crazy Bitcoin laser maxi".to_string(),
        "as a Ape".to_string(),
        "as SNES Pixel Art".to_string(),
        "as Modern Art Painting".to_string(),
        "anthropomorphic animals".to_string(),
        "rap album cover".to_string(),
        "outrun synthwave".to_string(),
        "propaganda poster".to_string(), // "newspaper".to_string(),
                                         // "fun".to_string(),
                                         // "beginbot as a service".to_string(),
                                         // "in a jail line up".to_string(),
                                         // "in an elon musk rocket ship on his way to mars".to_string(),
    ];
    let mut rng = rand::thread_rng();
    let selected_choice = choices.choose(&mut rng).unwrap();
    selected_choice.to_string()
}

// =================================================

use fal_rust::{
    client::{ClientCredentials, FalClient},
    utils::download_image,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ImageResult {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    images: Vec<ImageResult>,
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fal() {
        // So here is as silly place I can run fal
         let client = FalClient::new(ClientCredentials::from_env());

        let res = client
            .run(
                "fal-ai/stable-cascade",
                serde_json::json!({
                    "prompt": "A large waterfall in the middle of a volcano, surrounded by lush greenery and children's playground equipment.",
                }),
            )
            .await
            .unwrap();

        let output: Output = res.json::<Output>().await.unwrap();

        let url = output.images[0].url.clone();
        let filename = url.split('/').last().unwrap();

        download_image(&url, format!("{}/{}", "images", filename).as_str())
            .await
            .unwrap();
    }

    #[test]
    fn test_screenshot_variation() {
        let _screenshot_prompt = generate_random_prompt();
        //assert_eq!(screenshot_prompt,"");
    }
}
