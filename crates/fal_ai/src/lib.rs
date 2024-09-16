use bytes::Bytes;

use anyhow::anyhow;
use anyhow::{Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use mime_guess::MimeGuess;
use regex::Regex;
use reqwest::Client;
use rodio::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use std::path::Path;
use tokio::time::{sleep, Duration};

// Which do I need?
// use std::fs::File;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;

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

pub async fn handle_fal_commands(
    _sink: &Sink,
    splitmsg: Vec<String>,
) -> Result<()> {
    let command = splitmsg[0].as_str();

    match command {
        "!talk" => {
            let fal_image_file_path = "green_prime.png";
            let fal_audio_file_path =
                "TwitchChatTTSRecordings/1700109062_siifr_neo.wav";

            let video_bytes =
                sync_lips_to_voice(fal_image_file_path, fal_audio_file_path)
                    .await?;

            let video_path = "./prime.mp4";
            tokio::fs::write(&video_path, &video_bytes).await?;
            println!("Video saved to {}", video_path);
        }

        _ => {}
    };

    Ok(())
}

pub async fn sync_lips_to_voice(
    image_file_path: &str,
    audio_file_path: &str,
) -> Result<Bytes> {
    let fal_source_image_data_uri =
        fal_encode_file_as_data_uri(image_file_path).await?;

    let fal_driven_audio_data_uri =
        fal_encode_file_as_data_uri(audio_file_path).await?;

    // Submit the request to fal and handle the result
    match fal_submit_sadtalker_request(
        &fal_source_image_data_uri,
        &fal_driven_audio_data_uri,
    )
    .await
    {
        Ok(fal_result) => {
            println!("fal Result: {}", fal_result);

            // Parse the fal_result JSON
            let fal_result_json: serde_json::Value =
                serde_json::from_str(&fal_result)?;
            // Extract the video URL
            if let Some(video_obj) = fal_result_json.get("video") {
                if let Some(url_value) = video_obj.get("url") {
                    if let Some(url) = url_value.as_str() {
                        // Download the video
                        let client = reqwest::Client::new();
                        let resp = client.get(url).send().await?;
                        let video_bytes = resp.bytes().await?;

                        // Ensure the directory exists
                        tokio::fs::create_dir_all("./tmp/fal_videos").await?;

                        // Generate a timestamp for the filename
                        let timestamp = chrono::Utc::now().timestamp();

                        // Save the video to ./tmp/fal_videos
                        let video_path =
                            format!("./tmp/fal_videos/{}.mp4", timestamp);
                        tokio::fs::write(&video_path, &video_bytes).await?;
                        println!("Video saved to {}", video_path);

                        return Ok(video_bytes);
                        // This probably shouldn't happen in here
                        // let video_path = "./prime.mp4";
                        // tokio::fs::write(&video_path, &video_bytes).await?;
                        // println!("Video saved to {}", video_path);
                    } else {
                        eprintln!("Error: 'url' is not a string");
                    }
                } else {
                    eprintln!("Error: 'url' field not found in 'video' object");
                }
            } else {
                eprintln!("Error: 'video' field not found in fal_result");
            }
        }
        Err(e) => {
            eprintln!("fal Error: {}", e);
        }
    }
    return Err(anyhow!("Error: fal request failed"));
}

async fn process_images(
    timestamp: &str,
    json_path: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    // Read the JSON file asynchronously
    let json_data = tokio::fs::read_to_string(json_path).await?;

    // Parse the JSON data into the FalData struct
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

            // Construct the filename using the timestamp and extension
            let filename =
                format!("tmp/fal_images/{}.{}", timestamp, extension);

            // Save the image bytes to a file asynchronously
            let mut file =
                File::create(&filename).await.with_context(|| {
                    format!("Error creating file: {}", filename)
                })?;
            file.write_all(&image_bytes).await.with_context(|| {
                format!("Error writing to file: {}", filename)
            })?;

            // **New Code Start**
            // Also save the image to "./tmp/dalle-1.png"
            let additional_filename = "./tmp/dalle-1.png";
            let mut additional_file =
                File::create(additional_filename).await.with_context(|| {
                    format!("Error creating file: {}", additional_filename)
                })?;
            additional_file.write_all(&image_bytes).await.with_context(
                || format!("Error writing to file: {}", additional_filename),
            )?;
            println!("Also saved to {}", additional_filename);
            // **New Code End**

            // Optionally save the image to an additional location
            if let Some(extra_folder) = extra_save_folder {
                let extra_filename =
                    format!("{}/{}.{}", extra_folder, timestamp, extension);
                let mut extra_file =
                    File::create(&extra_filename).await.with_context(|| {
                        format!("Error creating file: {}", extra_filename)
                    })?;
                extra_file.write_all(&image_bytes).await.with_context(
                    || format!("Error writing to file: {}", extra_filename),
                )?;
            }

            println!("Saved {}", filename);
        } else {
            eprintln!("Invalid data URL for image at index {}", index);
        }
    }

    Ok(())
}

pub async fn create_turbo_image_in_folder(
    prompt: String,
    suno_save_folder: &String,
) -> Result<()> {
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

    // This is not the folder
    // let save_folder = "tmp/fal_images";
    let _ = process_images(
        &timestamp.to_string(),
        &json_path,
        Some(&suno_save_folder),
    )
    .await;

    Ok(())
}

// This is too specific
pub async fn create_turbo_image(prompt: String) -> Result<()> {
    // Can I move this into it's own function that takes a prompt?
    // So here is as silly place I can run fal
    let client = FalClient::new(ClientCredentials::from_env());

    // let model = "fal-ai/stable-cascade/sote-diffusion";
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
    let _ = process_images(&timestamp.to_string(), &json_path, None).await;

    Ok(())
}

// Function to submit the request to the fal 'sadtalker' model
async fn fal_submit_sadtalker_request(
    fal_source_image_data_uri: &str,
    fal_driven_audio_data_uri: &str,
) -> Result<String> {
    let fal_client = FalClient::new(ClientCredentials::from_env());

    // Prepare the JSON payload specific to fal
    // let fal_arguments = json!({
    //     "source_image_url": fal_source_image_data_uri,
    //     "driven_audio_url": fal_driven_audio_data_uri,
    // });

    // Send a POST request to the fal 'sadtalker' API endpoint
    let fal_response = fal_client
        .run(
            "fal-ai/sadtalker",
            serde_json::json!({
                "source_image_url": fal_source_image_data_uri,
                "driven_audio_url": fal_driven_audio_data_uri,
            }),
        )
        .await
        .unwrap();

    // Check if the request was successful
    if fal_response.status().is_success() {
        // Retrieve the response body as text
        let fal_result = fal_response.text().await?;
        Ok(fal_result)
    } else {
        // Return an error with the status code
        Err(anyhow!(format!(
            "fal request failed with status: {}",
            fal_response.status()
        )))
    }
}

async fn fal_encode_file_as_data_uri(file_path: &str) -> Result<String> {
    // Open the file asynchronously
    let mut fal_file = File::open(file_path).await?;
    let mut fal_file_data = Vec::new();

    // Read the entire file into the buffer
    fal_file.read_to_end(&mut fal_file_data).await?;

    // Encode the file data to Base64
    let fal_encoded_data = general_purpose::STANDARD.encode(&fal_file_data);

    // Convert the encoded data to a String
    let fal_encoded_data_string =
        String::from_utf8(fal_encoded_data.into_bytes())?;

    // Guess the MIME type based on the file extension
    let fal_mime_type = MimeGuess::from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    // Create the data URI for fal
    let fal_data_uri =
        format!("data:{};base64,{}", fal_mime_type, fal_encoded_data_string);

    Ok(fal_data_uri)
}
