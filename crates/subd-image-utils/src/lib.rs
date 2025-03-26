use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use bytes::Bytes;
use mime_guess::MimeGuess;
use regex::Regex;
use reqwest::redirect::Policy;
use reqwest::Client as ReqwestClient;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use tokio::io::AsyncReadExt;

pub async fn download_video(url: &str) -> Result<Bytes> {
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

pub async fn download_image_to_vec(
    url: &str,
    download_path: Option<&str>,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .redirect(Policy::default())
        .build()
        .expect("Failed to create client");

    println!("\tCall out to URL {} to download image to", url);
    let image_data = client.get(url).send().await?.bytes().await?.to_vec();

    if let Some(path) = download_path {
        println!("\tSaving File: {}", path);
        File::create(path).and_then(|mut f| f.write_all(&image_data))?;
    }
    Ok(image_data)
}

pub fn encode_image(image_path: &str) -> io::Result<String> {
    let mut file = File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);
    Ok(b64)
}

pub async fn encode_file_as_data_uri(file_path: &str) -> Result<String> {
    let mut file = tokio::fs::File::open(file_path)
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

pub fn parse_data_url(data_url: &str) -> Result<(&str, &str)> {
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

pub async fn get_image_bytes(url: &str) -> Result<Vec<u8>> {
    if url.starts_with("data:") {
        let (_mime_type, base64_data) = parse_data_url(url)
            .with_context(|| "Invalid data URL for image".to_string())?;
        let image_bytes =
            general_purpose::STANDARD.decode(base64_data).with_context(
                || "Failed to decode base64 data for image".to_string(),
            )?;
        Ok(image_bytes)
    } else {
        let image_bytes = download_image_to_vec(url, None).await?;
        Ok(image_bytes)
    }
}
