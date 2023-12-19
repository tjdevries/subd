use anyhow::Result;
use reqwest;
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::redirect::Policy;


pub async fn download_image(
    url: String,
    download_path: String,
) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .redirect(Policy::default())
        .build()
        .expect("Failed to create client");
    
    let image_data = client.get(url.clone())
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?
        .to_vec();

    let _ = File::create(download_path.clone())
        .map_err(|e| e.to_string())
        .and_then(|mut f| f.write_all(&image_data).map_err(|e| e.to_string()));

    Ok(image_data)
}

pub fn encode_image(image_path: &str) -> io::Result<String> {
    let mut file = File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);
    Ok(b64)
}

