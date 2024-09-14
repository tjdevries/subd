use crate::openai::dalle;
use std::io::Write;
use std::fs::File;
use std::path::Path;
use anyhow::anyhow;
use base64::engine;
use base64::{engine::general_purpose, Engine as _};
use crate::telephone;
use anyhow::{Context, Result};
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

#[derive(Debug, Serialize, Deserialize, Default)]
struct ImageResult {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    images: Vec<ImageResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FalResponse {
    images: Vec<TurboImageResult>,
    seed: u64,
    // num_inference_steps: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TurboImageResult {

    // That fact this is string is wrong?
    url: String,
    width: u32,
    height: u32,
}

use std::error::Error;
use regex::Regex;
// use base64::decode;


// ================

#[derive(Deserialize)]
struct Image {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    content_type: Option<String>,
}

#[derive(Deserialize)]
struct Data {
    images: Vec<Image>,
    // Other fields can be added here if needed
}

async fn process_images(timestamp: &str, json_path: &str) -> Result<()> {
    // Read the JSON file asynchronously

    // need to take the json_path name and extract out the timestamp
    let json_data = tokio::fs::read_to_string(json_path).await?;

    // Parse the JSON data into the Data struct
    let data: Data = serde_json::from_str(&json_data)?;

    // Regex to match data URLs
    let data_url_regex = Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;

    for (index, image) in data.images.iter().enumerate() {
        // Match the data URL and extract MIME type and base64 data
        if let Some(captures) = data_url_regex.captures(&image.url) {
            let mime_type = captures.name("mime").unwrap().as_str();
            let base64_data = captures.name("data").unwrap().as_str();

            // Decode the base64 data
            let image_bytes = general_purpose::STANDARD.decode(base64_data)?;

            // Determine the file extension based on the MIME type
            let extension = match mime_type {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                _ => "bin", // Default to binary if unknown type
            };

            // We might want to look for an ID here or make sure we are using the same json
            let filename = format!("tmp/fal_images/{}.{}", timestamp, extension);

            // Save the image bytes to a file
            let mut file = File::create(&filename)?;
            file.write_all(&image_bytes)?;

            
            let filename = format!("./tmp/dalle-1.png");
            let _ = File::create(&Path::new(&filename))
                .map(|mut f| f.write_all(&image_bytes))
                .with_context(|| format!("Error creating: {}", filename))?;

            println!("Saved {}", filename);
        } else {
            eprintln!("Invalid data URL for image at index {}", index);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_parsing_fal() {
        // Saved w/ Text
        // let tmp_file_path = "tmp/fal_responses/1726345706.json";
        //
        // Saved with bytes
        let timestamp = "1726347150";
        let tmp_file_path = format!("tmp/fal_responses/{}.json", timestamp);
        
        process_images(&timestamp, &tmp_file_path).await.unwrap();
            
        // Read the JSON file
        // let json_content = tokio::fs::read_to_string(&tmp_file_path).await.unwrap();
        // 
        // // Parse the JSON into a generic Value
        // let parsed: FalResponse = serde_json::from_str(&json_content).unwrap();
        // // let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        // 
        // println!("\n\n--------");
        // for o in parsed.images {
        //     let image_data = general_purpose::STANDARD
        //         .decode(o.url);
        //         // .map_err(|e| anyhow!(e.to_string()));
        //         // .and_then(|v| Ok(v));
        //     //
        //     println!("Image Data {:?}", image_data);
        //     
        //     // let archive_file = format!("tmp/fal_images/{}.png", chrono::Utc::now().timestamp());
        //     // let _ = File::create(&Path::new(&archive_file))
        //     //     .map(|mut f| f.write_all(&image_data))
        //     //         .with_context(|| format!("Error creating: {}", archive_file)).unwrap();
        //     }
        //     
        // // You can now access the parsed data
        // // println!("Parsed JSON: {:?}", parsed);
        // 
        // // If you want to parse into a specific struct, you can do:
        // // let output: Output = serde_json::from_str(&json_content).unwrap();
        // // println!("Parsed Output: {:?}", output);
    }
    
    #[tokio::test]
    async fn test_fal() {
        // So here is as silly place I can run fal
         let client = FalClient::new(ClientCredentials::from_env());

        // let model = "fal-ai/stable-cascade";
        let model = "fal-ai/fast-turbo-diffusion";
        
        let res = client
            .run(
                model,
                serde_json::json!({
                    "prompt": "A Orange lego Cat eating lego lasagna",
                }),
            )
            .await
            .unwrap();

        let raw_json = res.bytes().await.unwrap();
        let timestamp = chrono::Utc::now().timestamp();
        let json_path = format!("tmp/fal_responses/{}.json", timestamp);
        tokio::fs::write(&json_path, &raw_json).await.unwrap();
        let _ = process_images(&timestamp.to_string(), &json_path).await;
        

        // so now we could just process the image here

        // let output: Output = serde_json::from_str(&raw_json).unwrap();

        // So we don't have URL here
        // let url = output.images[0].url.clone();
        // let filename = url.split('/').last().unwrap();

        // download_image(&url, format!("{}/{}", "images", filename).as_str())
        //     .await
        //     .unwrap();
    }

    #[test]
    fn test_screenshot_variation() {
        let _screenshot_prompt = generate_random_prompt();
        //assert_eq!(screenshot_prompt,"");
    }
}
