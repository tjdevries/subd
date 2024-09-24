use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use once_cell::sync::Lazy;
use regex::bytes::Regex;
use std::str;
use tokio::fs::create_dir_all;

static DATA_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)").unwrap()
});

pub fn extract_image_data(data_url: &str) -> Result<(Vec<u8>, String)> {
    let data_url_bytes = data_url.as_bytes();

    if let Some(captures) = DATA_URL_REGEX.captures(data_url_bytes) {
        let mime_match = captures.name("mime").unwrap();
        let base64_match = captures.name("data").unwrap();

        let mime_type = str::from_utf8(
            &data_url_bytes[mime_match.start()..mime_match.end()],
        )?;
        let base64_data = str::from_utf8(
            &data_url_bytes[base64_match.start()..base64_match.end()],
        )?;

        let image_bytes = general_purpose::STANDARD.decode(base64_data)?;

        let extension = match mime_type {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            _ => "bin",
        };

        Ok((image_bytes, extension.to_string()))
    } else {
        Err(anyhow!("Invalid data URL"))
    }
}

/// Saves image bytes to the specified file path.
pub async fn save_raw_bytes(filename: &str, raw_bytes: &[u8]) -> Result<()> {
    let dir = std::path::Path::new(filename).parent().unwrap();
    create_dir_all(dir).await?;

    tokio::fs::write(&filename, raw_bytes)
        .await
        .with_context(|| format!("Failed to write video to {}", filename))?;
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
