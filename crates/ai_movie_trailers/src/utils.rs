use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use mime_guess::MimeGuess;
use regex::Regex;
use reqwest::Client as ReqwestClient;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};

//pub async fn process_fal_images_from_json(
//    raw_json: &[u8],
//    timestamp: &str,
//    extra_save_folder: Option<&str>,
//) -> Result<()> {
//    let images = parse_images_from_json(raw_json)?;
//    for (index, image) in images.into_iter().enumerate() {
//        process_image(index, &image, timestamp, extra_save_folder).await?;
//    }
//    Ok(())
//}
//
async fn process_image(
    index: usize,
    image: &serde_json::Value,
    timestamp: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    let url = match image["url"].as_str() {
        Some(url) => url,
        None => {
            eprintln!("Failed to find image URL for image at index {}", index);
            return Ok(());
        }
    };

    let image_bytes = get_image_bytes(url, index).await?;
    let extension = "png"; // Assuming PNG
    save_image_bytes(&image_bytes, timestamp, extension, extra_save_folder)
        .await?;
    Ok(())
}

fn parse_images_from_json(raw_json: &[u8]) -> Result<Vec<serde_json::Value>> {
    let data: serde_json::Value = serde_json::from_slice(raw_json)?;
    data["images"]
        .as_array()
        .cloned()
        .ok_or_else(|| anyhow!("Failed to extract images from JSON"))
}

async fn get_image_bytes(url: &str, index: usize) -> Result<Vec<u8>> {
    if url.starts_with("data:") {
        let (_mime_type, base64_data) =
            parse_data_url(url).with_context(|| {
                format!("Invalid data URL for image at index {}", index)
            })?;
        let image_bytes = general_purpose::STANDARD
            .decode(base64_data)
            .with_context(|| {
                format!(
                    "Failed to decode base64 data for image at index {}",
                    index
                )
            })?;
        Ok(image_bytes)
    } else {
        let image_bytes = download_image(url).await?.to_vec();
        Ok(image_bytes)
    }
}

fn parse_data_url(data_url: &str) -> Result<(&str, &str)> {
    let data_url_regex =
        Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;
    let captures = data_url_regex
        .captures(data_url)
        .ok_or_else(|| anyhow!("Invalid data URL"))?;
    let mime_type = captures
        .name("mime")
        .ok_or_else(|| anyhow!("Missing mime type in data URL"))?
        .as_str();
    let base64_data = captures
        .name("data")
        .ok_or_else(|| anyhow!("Missing base64 data in data URL"))?
        .as_str();
    Ok((mime_type, base64_data))
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

pub fn extract_video_url_from_fal_result(fal_result: &str) -> Result<String> {
    let fal_result_json: serde_json::Value = serde_json::from_str(fal_result)?;
    fal_result_json
        .get("video")
        .and_then(|video| video.get("url"))
        .and_then(|url| url.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Failed to extract video URL from FAL result"))
}
