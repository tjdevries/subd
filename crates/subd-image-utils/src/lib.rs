use anyhow::{Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use bytes::Bytes;
use mime_guess::MimeGuess;
use reqwest;
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
    url: String,
    download_path: Option<String>,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .redirect(Policy::default())
        .build()
        .expect("Failed to create client");

    println!("\tCall out to URL {} to download image to", url.clone());
    let image_data = client
        .get(url.clone())
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();

    if let Some(path) = download_path {
        println!("\tSaving File: {}", path);
        File::create(path.clone())
            .and_then(|mut f| f.write_all(&image_data))?;
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
