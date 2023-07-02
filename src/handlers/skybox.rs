extern crate reqwest;
extern crate serde; extern crate serde_json;
use anyhow::Result;
use std::env;
use async_trait::async_trait;
use chrono::prelude::*;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use subd_types::Event;
use tokio::sync::broadcast;
use tokio;

#[allow(dead_code)]
pub struct Skybox {
    pub name: String,
}

#[allow(dead_code)]
pub struct SkyboxHandler {
    pub obs_client: OBSClient,
}

#[allow(dead_code)]
pub struct SkyboxRemixHandler {
    pub obs_client: OBSClient,
}


#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::SkyboxRequest(msg) => msg,
                _ => continue,
            };


            println!("\n\n\tSkyboxHandler: {:?}", msg);
            // crate::obs_hotkeys::trigger_hotkey(&msg.hotkey, &self.obs_client).await?;
        }
    }
}

// ============================================================================================
// ============================================================================================
// ============================================================================================
// ============================================================================================
//
// CHAT GPT Generated Code, BE CAREFUL

#[allow(dead_code)]
static SKYBOX_REMIX_URL: &str = "https://backend.blockadelabs.com/api/v1/skybox";
static SKYBOX_IMAGINE_URL: &str = "https://backend.blockadelabs.com/api/v1/imagine";

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterRequest {
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkyboxStyle {
    pub id: i32,
    pub name: String,
    pub max_char: String,
    pub image: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemixRequestResponse {
    pub id: i32,
    pub obfuscated_id: String,
    pub user_id: i32,
    pub title: String,
    pub prompt: String,
    pub username: String,
    pub status: String,
    pub queue_position: i32,
    pub file_url: String,
    pub thumb_url: String,
    pub depth_map_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
    pub pusher_channel: String,
    pub pusher_event: String,
    pub _type: String,
    pub skybox_style_id: i32,
    pub skybox_id: i32,
    pub skybox_style_name: String,
    pub skybox_name: String,
}

#[derive(Debug, Serialize, Deserialize)] pub struct GeneratorData {
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

// TODO: add the logic for this later
#[allow(dead_code)]
#[allow(unused_variables)]
fn find_style_id(words: Vec<&str>) -> i32 {
    // What is a good default style ID
    return 1
}

#[allow(dead_code)]
async fn request_image(prompt: String) -> io::Result<String> {
    let skybox_imagine_url = env::var("SKYBOX_IMAGINE_URL").unwrap();
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();
    let requests_url = format!("{}/requests?api_key={}", skybox_imagine_url, skybox_api_key);

    let prompt = prompt.trim_start().to_string();
    let words: Vec<&str> = prompt.split_whitespace().collect();

    let skybox_style_id = find_style_id(words); // Assuming find_style_id function exists

    println!("Generating Skybox w/ Custom Skybox ID: {}", skybox_style_id);
    
    let post_body = json!({
        "prompt": prompt,
        "generator": "stable-skybox",
        "skybox_style_id": skybox_style_id,
    });

    let client = Client::new();
    let resp = client.post(&requests_url)
        .json(&post_body)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let bytes = body.as_bytes();

    let t = Utc::now();
    let response_filepath = format!("/tmp/{}.json", t);

    let mut file = File::create(response_filepath.clone())?;
    file.write_all(bytes)?;

    Ok(response_filepath)
}

#[allow(dead_code)]
async fn request_status(id: &str) -> Result<Response> {
    let skybox_api_key: String = std::env::var("SKYBOX_API_KEY").unwrap();
    let url = format!("{}/requests/{}?api_key={}", SKYBOX_IMAGINE_URL, id, skybox_api_key);

    let resp = reqwest::get(&url).await?;
    let body = resp.text().await?;

    let parsed_response: OuterRequest = serde_json::from_str(&body)?;

    Ok(parsed_response.response)
}


// async fn remix(remix_id: i32, style_id: i32, prompt: &str) -> Result<String, Box<dyn Error>> {
//     // Perform HTTP POST request here...
//     let requests_url = format!("{}?api_key={}", SKYBOX_REMIX_URL, SKYBOX_API_KEY);
//     // Generate the post body and perform the HTTP request...
//
//     let response_body = reqwest::Client::new().post(&requests_url).json(&map).send().await?;
//     let body = response_body.text().await?;
//
//     // Write to file here...
//
//     let skybox_remix_response_file_path = "/home/begin/code/subd/tmp/skybox_archive";
//     Ok(skybox_remix_response_file_path)
// }
