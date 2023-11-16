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


            request_skybox(request.msg).await?;

            // TODO: we will need to trigger the skybox OBS source
            // to refresh, after we get an updated Skybox
            // AND generate a fresh HTML page using pannellum
        }
    }
}

#[allow(dead_code)]
async fn request_skybox(prompt: String) -> io::Result<String> {
    
    // let skybox_api_key = match env::var_os("SKYBOX_API_KEY") {
    //         Some(v) => v.into_string().unwrap(),
    //         None => panic!("$SKYBOX_API_KEY is not set")
    // };
    // This API Key could be blank
    // let skybox_api_key: String = String::from("3c4bDk5l777GwoXdULwFuB6bqwYJwr1fDN9GL3bhw8XQ4W7Vv7RiV0JAxH5c");
    // // let skybox_api_key: String = String::from("IVgnrZpVpTYbBzgW0Lk3vJIRvNOQuxnYHOw5n1HI9O8AMnib3gdAhPFUQkak");
    
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();
    // TODO: update this to not pass the API KEY through the request
    // https://backend.blockadelabs.com/api/v1/skybox
    let requests_url = format!("{}?api_key={}", SKYBOX_REMIX_URL, skybox_api_key);

    // So this doesn't work right now because we don't a have a working subscription
    // println!("Skybox API URL: {}", requests_url);
    // return Ok(requests_url);

    // Do we need to trim start
    // or should this done before i'ts passed
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
    let resp = client.post(&requests_url)
        .json(&post_body)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let bytes = body.as_bytes();

    let t = Utc::now();
    // so I think this path should be relative
    let response_filepath = format!("./tmp/skybox_{}.json", t);

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
async fn request_status(id: &str) -> Result<Response> {
    // let skybox_api_key: String = std::env::var("SKYBOX_API_KEY").unwrap();
    let skybox_api_key: String = String::from("3c4bDk5l777GwoXdULwFuB6bqwYJwr1fDN9GL3bhw8XQ4W7Vv7RiV0JAxH5c");
    // let skybox_api_key: String = String::from("IVgnrZpVpTYbBzgW0Lk3vJIRvNOQuxnYHOw5n1HI9O8AMnib3gdAhPFUQkak");
    let url = format!("{}/requests/{}?api_key={}", SKYBOX_IMAGINE_URL, id, skybox_api_key);

    println!("URL: {}", url);
    // x-api-key

    let client = reqwest::Client::new();
    let resp = client.get(&url)
        // .header("x-api-key", skybox_api_key)
        .send()
        .await?;
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
