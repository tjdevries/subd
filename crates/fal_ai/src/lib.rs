use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use chrono::Utc;
use mime_guess::MimeGuess;
use regex::Regex;
use serde::Deserialize;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use fal_rust::client::{ClientCredentials, FalClient};
use reqwest::Client as ReqwestClient;

#[derive(Deserialize)]
struct FalImage {
    url: String,
    _width: Option<u32>,
    _height: Option<u32>,
    _content_type: Option<String>,
}

#[derive(Deserialize)]
struct FalData {
    images: Vec<FalImage>,
}

pub async fn sync_lips_to_voice(
    image_file_path: &str,
    audio_file_path: &str,
) -> Result<Bytes> {
    println!("\tEncoding Image File Info");
    let fal_source_image_data_uri =
        fal_encode_file_as_data_uri(image_file_path).await?;

    println!("\tEncoding Audio File Info");
    let fal_driven_audio_data_uri =
        fal_encode_file_as_data_uri(audio_file_path).await?;

    println!("\tTalking to FAL");
    let fal_result = fal_submit_sadtalker_request(
        &fal_source_image_data_uri,
        &fal_driven_audio_data_uri,
    )
    .await?;
    println!("FAL Result: {}", fal_result);

    let video_url = extract_video_url_from_fal_result(&fal_result)?;
    let video_bytes = download_video(&video_url).await?;

    let timestamp = Utc::now().timestamp();
    let video_path = format!("./tmp/fal_videos/{}.mp4", timestamp);
    create_dir_all("./tmp/fal_videos").await?;
    tokio::fs::write(&video_path, &video_bytes)
        .await
        .with_context(|| format!("Failed to write video to {}", video_path))?;
    println!("Video saved to {}", video_path);

    Ok(video_bytes)
}

pub async fn create_turbo_image_in_folder(
    prompt: String,
    suno_save_folder: &str,
) -> Result<()> {
    let client = FalClient::new(ClientCredentials::from_env());
    let model = "fal-ai/stable-cascade";
    println!("\tCreating image with model: {}", model);

    let res = client
        .run(
            model,
            serde_json::json!({
                "prompt": prompt,
                "image_size": "landscape_16_9",
            }),
        )
        .await
        .map_err(|e| anyhow!("Failed to run FAL Client: {:?}", e))?;

    let raw_json = res
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from FAL response")?;

    let timestamp = Utc::now().timestamp();
    let json_path = format!("tmp/fal_responses/{}.json", timestamp);
    create_dir_all("tmp/fal_responses").await?;
    tokio::fs::write(&json_path, &raw_json)
        .await
        .with_context(|| format!("Failed to write JSON to {}", json_path))?;

    process_fal_images_from_json(
        &raw_json,
        &timestamp.to_string(),
        Some(suno_save_folder),
    )
    .await?;

    Ok(())
}

pub async fn create_video_from_image(image_file_path: &str) -> Result<()> {
    let fal_source_image_data_uri =
        fal_encode_file_as_data_uri(image_file_path).await?;
    let client = FalClient::new(ClientCredentials::from_env());

    let response = client
        .run(
            "fal-ai/stable-video",
            serde_json::json!({ "image_url": fal_source_image_data_uri }),
        )
        .await
        .map_err(|e| anyhow!("Failed to run client: {:?}", e))?;

    let body = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    if let Some(url) = json["video"]["url"].as_str() {
        let video_bytes = download_video(url).await?;
        let timestamp = Utc::now().timestamp();
        let filename = format!("tmp/fal_videos/{}.mp4", timestamp);
        create_dir_all("tmp/fal_videos").await?;
        tokio::fs::write(&filename, &video_bytes)
            .await
            .with_context(|| {
                format!("Failed to write video to {}", filename)
            })?;
        println!("Video saved to: {}", filename);
    } else {
        return Err(anyhow!("Failed to extract video URL from JSON"));
    }

    Ok(())
}

pub async fn create_turbo_image(prompt: String) -> Result<()> {
    let client = FalClient::new(ClientCredentials::from_env());
    let model = "fal-ai/fast-sdxl";
    println!("\t\tCreating image with model: {}", model);

    let res = client
        .run(
            model,
            serde_json::json!({
                "prompt": prompt,
                "image_size": "landscape_16_9",
            }),
        )
        .await
        .map_err(|e| anyhow!("Error running Fal Client: {:?}", e))?;

    let raw_json = res
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from response")?;

    let timestamp = Utc::now().timestamp();
    let json_path = format!("tmp/fal_responses/{}.json", timestamp);
    create_dir_all("tmp/fal_responses").await?;
    tokio::fs::write(&json_path, &raw_json)
        .await
        .with_context(|| format!("Failed to write JSON to {}", json_path))?;

    process_fal_images_from_json(&raw_json, &timestamp.to_string(), None)
        .await?;

    Ok(())
}

async fn process_fal_images_from_json(
    raw_json: &[u8],
    timestamp: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    let data: serde_json::Value = serde_json::from_slice(raw_json)?;

    if let Some(images) = data["images"].as_array() {
        for (index, image) in images.iter().enumerate() {
            if let Some(url) = image["url"].as_str() {
                let image_bytes = if url.starts_with("data:") {
                    if let Some((_mime_type, base64_data)) =
                        parse_data_url(url)?
                    {
                        general_purpose::STANDARD
                            .decode(base64_data)
                            .with_context(|| format!("Failed to decode base64 data for image at index {}", index))?
                    } else {
                        eprintln!(
                            "Invalid data URL for image at index {}",
                            index
                        );
                        continue;
                    }
                } else {
                    download_image(url).await?.to_vec()
                };

                let extension = "png"; // Assuming PNG
                save_image_bytes(
                    &image_bytes,
                    timestamp,
                    extension,
                    extra_save_folder,
                )
                .await?;
            } else {
                eprintln!(
                    "Failed to find image URL for image at index {}",
                    index
                );
            }
        }
    } else {
        return Err(anyhow!("Failed to extract images from JSON"));
    }

    Ok(())
}

async fn save_image_bytes(
    image_bytes: &[u8],
    timestamp: &str,
    extension: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    let filename = format!("tmp/fal_images/{}.{}", timestamp, extension);
    create_dir_all("tmp/fal_images").await?;
    tokio::fs::write(&filename, image_bytes)
        .await
        .with_context(|| format!("Error writing to file: {}", filename))?;

    let additional_filename = "./tmp/dalle-1.png";
    tokio::fs::write(additional_filename, image_bytes)
        .await
        .with_context(|| {
            format!("Error writing to file: {}", additional_filename)
        })?;

    if let Some(extra_folder) = extra_save_folder {
        let extra_filename =
            format!("{}/{}.{}", extra_folder, timestamp, extension);
        create_dir_all(extra_folder).await?;
        tokio::fs::write(&extra_filename, image_bytes)
            .await
            .with_context(|| {
                format!("Error writing to file: {}", extra_filename)
            })?;
    }

    println!("Saved {}", filename);
    Ok(())
}

async fn fal_encode_file_as_data_uri(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)
        .await
        .with_context(|| format!("Failed to open file: {}", file_path))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let encoded_data = general_purpose::STANDARD.encode(&buffer);
    let mime_type = MimeGuess::from_path(file_path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    Ok(format!("data:{};base64,{}", mime_type, encoded_data))
}

async fn fal_submit_sadtalker_request(
    fal_source_image_data_uri: &str,
    fal_driven_audio_data_uri: &str,
) -> Result<String> {
    let fal_client = FalClient::new(ClientCredentials::from_env());
    let response = fal_client
        .run(
            "fal-ai/sadtalker",
            serde_json::json!({
                "source_image_url": fal_source_image_data_uri,
                "driven_audio_url": fal_driven_audio_data_uri,
            }),
        )
        .await
        .map_err(|e| anyhow!("Error running sadtalker {:?}", e))?;

    if response.status().is_success() {
        response
            .text()
            .await
            .map_err(|e| anyhow!("Error getting text: {:?}", e))
    } else {
        Err(anyhow!(
            "FAL request failed with status: {}",
            response.status()
        ))
    }
}

fn parse_data_url(data_url: &str) -> Result<Option<(&str, &str)>> {
    let data_url_regex =
        Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;
    if let Some(captures) = data_url_regex.captures(data_url) {
        let mime_type = captures.name("mime").unwrap().as_str();
        let base64_data = captures.name("data").unwrap().as_str();
        Ok(Some((mime_type, base64_data)))
    } else {
        Ok(None)
    }
}

fn extract_video_url_from_fal_result(fal_result: &str) -> Result<String> {
    let fal_result_json: serde_json::Value = serde_json::from_str(fal_result)?;
    fal_result_json
        .get("video")
        .and_then(|video| video.get("url"))
        .and_then(|url| url.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Failed to extract video URL from FAL result"))
}

async fn download_video(url: &str) -> Result<Bytes> {
    let client = ReqwestClient::new();
    let resp = client.get(url).send().await.with_context(|| {
        format!("Failed to download video from URL: {}", url)
    })?;
    let video_bytes = resp
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from response")?;
    Ok(video_bytes)
}

async fn download_image(url: &str) -> Result<Bytes> {
    let client = ReqwestClient::new();
    let response = client.get(url).send().await.with_context(|| {
        format!("Failed to download image from URL: {}", url)
    })?;
    let image_bytes = response
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from image response")?;
    Ok(image_bytes)
}
