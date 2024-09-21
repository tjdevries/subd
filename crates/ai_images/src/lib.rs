use anyhow::{self, Result};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest;
use reqwest::redirect::Policy;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use std::process::Command;

pub mod image_generation;

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

// TODO: We need to create this resized directory, when it doesn't exist
// and make it it not absolute
pub fn resize_image(
    unique_identifier: String,
    filename: String,
) -> Result<(String, Vec<u8>)> {
    let output_path =
        format!("./tmp/screenshots/resized/{}", unique_identifier);
    Command::new("convert")
        .args(&[
            filename,
            "-resize".to_string(),
            "1280x720".to_string(),
            output_path.clone(),
        ])
        .status()?;
    let mut file = File::open(output_path.clone())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok((output_path, buffer))
}
