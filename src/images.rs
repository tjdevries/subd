use anyhow::{self, Result};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest;
use reqwest::redirect::Policy;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};

pub async fn download_image(
    url: String,
    download_path: String,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .redirect(Policy::default())
        .build()
        .expect("Failed to create client");

    println!("\tCalling Dalle: {}", download_path);
    let image_data = client
        .get(url.clone())
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();

    println!("\tCreating File: {}", download_path);
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
