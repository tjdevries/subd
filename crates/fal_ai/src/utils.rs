use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::fs::create_dir_all;

pub async fn parse_and_process_images_from_json(
    raw_json: &[u8],
    main_filename_pattern: &str,
    additional_filename_pattern: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    let images = parse_images_from_json(raw_json)?;
    let extension = "png"; // Assuming PNG

    for (index, image) in images.into_iter().enumerate() {
        let main_filename =
            format!("{}-{}.{}", main_filename_pattern, index, extension);
        let additional_filename =
            format!("{}-{}.{}", additional_filename_pattern, index, extension);
        let extra_filename = extra_save_folder.map(|folder| {
            format!(
                "{}/{}-{}.{}",
                folder, main_filename_pattern, index, extension
            )
        });

        process_image(
            index,
            &image,
            &main_filename,
            &additional_filename,
            extra_filename.as_deref(),
        )
        .await?;
    }
    Ok(())
}

async fn process_image(
    index: usize,
    image: &serde_json::Value,
    main_filename: &str,
    additional_filename: &str,
    extra_filename: Option<&str>,
) -> Result<()> {
    let url = match image["url"].as_str() {
        Some(url) => url,
        None => {
            eprintln!("Failed to find image URL for image at index {}", index);
            return Ok(());
        }
    };

    let image_bytes = subd_image_utils::get_image_bytes(url, index).await?;

    save_image_bytes(
        &image_bytes,
        main_filename,
        additional_filename,
        extra_filename,
    )
    .await?;
    Ok(())
}

async fn save_image_bytes(
    image_bytes: &[u8],
    main_filename: &str,
    additional_filename: &str,
    extra_filename: Option<&str>,
) -> Result<()> {
    // Ensure the parent directories exist for the main file
    if let Some(parent) = Path::new(main_filename).parent() {
        create_dir_all(parent).await?;
    }
    tokio::fs::write(main_filename, image_bytes)
        .await
        .with_context(|| format!("Error writing to file: {}", main_filename))?;

    // Ensure the parent directories exist for the additional file
    if let Some(parent) = Path::new(additional_filename).parent() {
        create_dir_all(parent).await?;
    }
    tokio::fs::write(additional_filename, image_bytes)
        .await
        .with_context(|| {
            format!("Error writing to file: {}", additional_filename)
        })?;

    // If an extra filename is provided, save the image there as well
    if let Some(extra_filename) = extra_filename {
        if let Some(parent) = Path::new(extra_filename).parent() {
            create_dir_all(parent).await?;
        }
        tokio::fs::write(extra_filename, image_bytes)
            .await
            .with_context(|| {
                format!("Error writing to file: {}", extra_filename)
            })?;
    }

    println!("Saved {}", main_filename);
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

fn parse_images_from_json(raw_json: &[u8]) -> Result<Vec<serde_json::Value>> {
    let data: serde_json::Value = serde_json::from_slice(raw_json)?;
    data["images"]
        .as_array()
        .cloned()
        .ok_or_else(|| anyhow!("Failed to extract images from JSON"))
}
