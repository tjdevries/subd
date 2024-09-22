use anyhow::{Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use bytes::Bytes;
use reqwest;
use reqwest::redirect::Policy;
use reqwest::Client as ReqwestClient;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};

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

pub async fn download_image(
    url: String,
    download_path: String,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .redirect(Policy::default())
        .build()
        .expect("Failed to create client");

    println!(
        "\tCall out to URL {} to download image to: {}",
        url.clone(),
        download_path
    );
    let image_data = client
        .get(url.clone())
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();

    println!("\tSaving File: {}", download_path);
    File::create(download_path.clone())
        .and_then(|mut f| f.write_all(&image_data))?;
    Ok(image_data)
}

pub fn encode_image(image_path: &str) -> io::Result<String> {
    let mut file = File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);
    Ok(b64)
}
