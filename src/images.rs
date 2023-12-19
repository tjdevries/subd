use anyhow::Result;
use reqwest;
use std::fs::File;
use std::io::Write;

pub async fn download_image(
    url: String,
    download_path: String,
) -> Result<Vec<u8>, String> {
    let image_data = reqwest::get(url.clone())
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
