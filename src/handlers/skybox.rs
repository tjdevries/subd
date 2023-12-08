extern crate reqwest;
extern crate serde;
extern crate serde_json;
use crate::skybox;
use anyhow::Result;
use async_trait::async_trait;
use chrono::prelude::*;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;

#[allow(dead_code)]
pub struct Skybox {
    pub name: String,
}

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
            let request = match event {
                Event::SkyboxRequest(msg) => msg,
                _ => continue,
            };


            // We don't just need to call request here
            // this event isn't just pure request
            // we need to start somewhere else
            println!("Attempting to Skybox");
            request_skybox(request.msg).await?;

            // Can I kick off another loop???

            // TODO: we will need to trigger the skybox OBS source
            // to refresh, after we get an updated Skybox
            // AND generate a fresh HTML page using pannellum
        }
    }
}

pub async fn check_skybox_status(id: i32) -> Result<()> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    // https://backend.blockadelabs.com/api/v1/skybox
    // https://api-documentation.blockadelabs.com/api/skybox.html#get-skybox-by-id
    let requests_url =
        format!("{}/{}?api_key={}", SKYBOX_STATUS_URL, id, skybox_api_key);
    let client = Client::new();
    let resp = client.get(&requests_url).send().await.unwrap();

    // We should be able to parse this into the StatusResponse
    let text = resp.text().await.unwrap();
    let parsed_response: skybox::SkyboxStatusResponse =
        serde_json::from_str(&text)?;

    println!("Parsed Response: {:?}", parsed_response);
    Ok(())
}

#[allow(dead_code)]
async fn request_skybox(prompt: String) -> io::Result<String> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    // https://backend.blockadelabs.com/api/v1/skybox
    let requests_url =
        format!("{}?api_key={}", SKYBOX_REMIX_URL, skybox_api_key);

    // Do we need to trim start
    // orjshould this done before i'ts passed
    let prompt = prompt.trim_start().to_string();

    // Why???
    let words: Vec<&str> = prompt.split_whitespace().collect();

    // This returns a static style currently
    let skybox_style_id = find_style_id(words);

    println!("Generating Skybox w/ Custom Skybox ID: {}", skybox_style_id);

    // return Ok(String::from("this a hack"));

    let post_body = json!({
        "prompt": prompt,
        // "generator": "stable-skybox",
        // "skybox_style_id": skybox_style_id,
    });

    let client = Client::new();
    let resp = client
        .post(&requests_url)
        .json(&post_body)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let bytes = body.as_bytes();

    let t = Utc::now();
    // so I think this path should be relative
    let response_filepath = format!("./tmp/skybox_{}.json", t);

    // I need a parsed out body, to save
    let mut file = File::create(response_filepath.clone())?;
    file.write_all(bytes)?;

    // We need to parse the response
    // we need to get the idea, and kick off aprocess that checks Skybox every X seconds
    // if our AI generated bg is done
    Ok(response_filepath)
}

// ============================================================================================
// ============================================================================================
// ============================================================================================
// ============================================================================================
//
// CHAT GPT Generated Code, BE CAREFUL

#[allow(dead_code)]
static SKYBOX_STATUS_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine/requests";

static SKYBOX_REMIX_URL: &str =
    "https://backend.blockadelabs.com/api/v1/skybox";

static SKYBOX_IMAGINE_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine";

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

// TODO: add the logic for this later
#[allow(dead_code)]
#[allow(unused_variables)]
fn find_style_id(words: Vec<&str>) -> i32 {
    // What is a good default style ID
    return 1;
}

// Why are we passing the API Key in the URL?
#[allow(dead_code)]
async fn request_status(id: &str) -> Result<Response> {
    let skybox_api_key: String = std::env::var("SKYBOX_API_KEY").unwrap();
    let url = format!(
        "{}/requests/{}?api_key={}",
        SKYBOX_IMAGINE_URL, id, skybox_api_key
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
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
