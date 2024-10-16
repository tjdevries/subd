use anyhow::{anyhow, Context, Result};
use std::str;
use tokio::fs::create_dir_all;

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
