use crate::constants;
use anyhow::anyhow;
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use events::EventHandler;
use obws::Client as OBSClient;
use regex::Regex;
use rodio::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

use fal_rust::{
    client::{ClientCredentials, FalClient},
    utils::download_image,
};

#[derive(Deserialize)]
struct FalImage {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    content_type: Option<String>,
}

#[derive(Deserialize)]
struct FalData {
    images: Vec<FalImage>,
    // Other fields can be added here if needed
}

pub struct FalHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for FalHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // loop {
        //    // We want to
        // }
        return Ok(());
    }
}

pub async fn handle_fal_commands(
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
    let command = splitmsg[0].as_str();

    match command {
        "!fal" => {}

        _ => {
            // This is wierd spot, but whatever, just testing
            let prompt = msg.contents;
            create_turbo_image(prompt).await?;
        }
    };

    Ok(())
}

async fn process_images(timestamp: &str, json_path: &str) -> Result<()> {
    // Read the JSON file asynchronously

    // need to take the json_path name and extract out the timestamp
    let json_data = tokio::fs::read_to_string(json_path).await?;

    // Parse the JSON data into the Data struct
    let data: FalData = serde_json::from_str(&json_data)?;

    // Regex to match data URLs
    let data_url_regex =
        Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;

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
            let filename =
                format!("tmp/fal_images/{}.{}", timestamp, extension);

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

async fn create_turbo_image(prompt: String) -> Result<()> {
    // Can I move this into it's own function that takes a prompt?
    // So here is as silly place I can run fal
    let client = FalClient::new(ClientCredentials::from_env());

    // let model = "fal-ai/stable-cascade";
    let model = "fal-ai/fast-turbo-diffusion";

    let res = client
        .run(
            model,
            serde_json::json!({
                "prompt": prompt,
                "image_size": "landscape_16_9",
            }),
        )
        .await
        .unwrap();

    let raw_json = res.bytes().await.unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    let json_path = format!("tmp/fal_responses/{}.json", timestamp);
    tokio::fs::write(&json_path, &raw_json).await.unwrap();
    let _ = process_images(&timestamp.to_string(), &json_path).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs;
    use serde_json::{json, Error, Value};

    #[tokio::test]
    async fn test_parsing_fal() {
        // Saved w/ Text
        // let tmp_file_path = "tmp/fal_responses/1726345706.json";
        //
        // Saved with bytes
        let timestamp = "1726347150";
        let tmp_file_path = format!("tmp/fal_responses/{}.json", timestamp);

        process_images(&timestamp, &tmp_file_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_fal() {
        let prompt = "Magical Cat wearing a wizard hat";
        create_turbo_image(prompt.to_string()).await;
    }
}
