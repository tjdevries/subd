use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::fs::create_dir_all;

pub async fn parse_and_process_images_from_json(
    raw_json: &[u8],
    main_filename_pattern: &str,
    additional_filename_pattern: &str,
    extra_save_folder: Option<&str>,
) -> Result<()> {
    // Parse images from the raw JSON data
    let images = parse_images_from_json(raw_json)?;
    let extension = "png"; // Assuming PNG as the image extension

    // Process each image
    for (index, image) in images.into_iter().enumerate() {
        // Construct filenames for saving the image
        let main_filename =
            format!("{}-{}.{}", main_filename_pattern, index, extension);
        let additional_filename =
            format!("{}.{}", additional_filename_pattern, extension);
        let extra_filename = extra_save_folder.map(|folder| {
            format!(
                "{}/{}-{}.{}",
                folder, main_filename_pattern, index, extension
            )
        });

        // Process and save the image
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
    // Extract the URL of the image from the JSON data
    if let Some(url) = image["url"].as_str() {
        // Retrieve the image bytes from the URL
        let image_bytes = subd_image_utils::get_image_bytes(url, index).await?;

        // Save the image bytes to the specified filenames
        save_image_bytes(
            &image_bytes,
            main_filename,
            additional_filename,
            extra_filename,
        )
        .await?;
    } else {
        eprintln!("Failed to find image URL for image at index {}", index);
    }
    Ok(())
}

async fn save_image_bytes(
    image_bytes: &[u8],
    main_filename: &str,
    additional_filename: &str,
    extra_filename: Option<&str>,
) -> Result<()> {
    // Save the image to the main filename
    save_image(image_bytes, main_filename).await?;

    // Save the image to the additional filename
    save_image(image_bytes, additional_filename).await?;

    // If an extra filename is provided, save the image there as well
    if let Some(extra_filename) = extra_filename {
        save_image(image_bytes, extra_filename).await?;
    }

    println!("Saved {}", main_filename);
    Ok(())
}

async fn save_image(image_bytes: &[u8], filename: &str) -> Result<()> {
    // Ensure the parent directories exist
    if let Some(parent) = Path::new(filename).parent() {
        create_dir_all(parent).await?;
    }
    // Write the image bytes to the file
    tokio::fs::write(filename, image_bytes)
        .await
        .with_context(|| format!("Error writing to file: {}", filename))?;
    Ok(())
}

pub fn extract_video_url_from_fal_result(fal_result: &str) -> Result<String> {
    // Parse the JSON string into a serde_json::Value
    let fal_result_json: serde_json::Value = serde_json::from_str(fal_result)?;

    // Navigate through the JSON to get the video URL
    fal_result_json
        .get("video")
        .and_then(|video| video.get("url"))
        .and_then(|url| url.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Failed to extract video URL from FAL result"))
}

fn parse_images_from_json(raw_json: &[u8]) -> Result<Vec<serde_json::Value>> {
    // Parse the raw JSON bytes into a serde_json::Value
    let data: serde_json::Value = serde_json::from_slice(raw_json)?;

    // Extract the array of images from the JSON data
    data["images"]
        .as_array()
        .cloned()
        .ok_or_else(|| anyhow!("Failed to extract images from JSON"))
}
