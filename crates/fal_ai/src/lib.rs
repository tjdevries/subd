use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use fal_rust::client::{ClientCredentials, FalClient};
use regex::Regex;
use serde::Deserialize;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

pub mod models;
pub mod utils;

#[derive(Deserialize)]
struct FalImage {
    url: String,
    _width: Option<u32>,
    _height: Option<u32>,
    _content_type: Option<String>,
}

#[derive(Deserialize)]
struct FalData {
    images: Vec<FalImage>,
}

/// A service for interacting with the FAL API.
pub struct FalService {
    client: FalClient,
}

impl FalService {
    /// Creates a new instance of `FalService`.
    pub fn new() -> Self {
        Self {
            client: FalClient::new(ClientCredentials::from_env()),
        }
    }

    /// Runs a model with the given parameters and returns the raw JSON response as bytes.
    async fn run_model_and_get_raw_json(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<bytes::Bytes> {
        let res =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        let raw_json = res.bytes().await.with_context(|| {
            format!("Failed to get bytes from model '{}'", model)
        })?;

        Ok(raw_json)
    }

    /// Runs a model with the given parameters and returns the response as JSON.
    async fn run_model_and_get_json(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let res =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        let body = res.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;
        Ok(json)
    }

    /// Runs a model with the given parameters and returns the response as text.
    async fn run_model_and_get_text(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<String> {
        let response =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        if response.status().is_success() {
            response.text().await.map_err(|e| {
                anyhow!("Error getting text from model '{}': {:?}", model, e)
            })
        } else {
            Err(anyhow!(
                "Request to model '{}' failed with status: {}",
                model,
                response.status()
            ))
        }
    }

    /// Saves the raw JSON response to a specified file path.
    async fn save_raw_json_response(
        &self,
        raw_json: &[u8],
        save_path: &str,
    ) -> Result<()> {
        // Ensure the directory exists
        let dir = std::path::Path::new(save_path).parent().unwrap();
        create_dir_all(dir).await?;

        // Write the JSON data to the file
        tokio::fs::write(&save_path, raw_json)
            .await
            .with_context(|| {
                format!("Failed to write JSON to {}", save_path)
            })?;

        Ok(())
    }

    /// Processes images from the raw JSON response and saves them to the specified directory.
    async fn process_images_from_json(
        &self,
        raw_json: &[u8],
        save_dir: &str,
    ) -> Result<()> {
        // Deserialize the JSON response into FalData struct
        let data: FalData = serde_json::from_slice(raw_json)?;

        // Regex to match data URLs in the image URLs
        let data_url_regex =
            Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;

        // Ensure the save directory exists
        create_dir_all(save_dir).await?;

        for (index, image) in data.images.iter().enumerate() {
            if let Some(captures) = data_url_regex.captures(&image.url) {
                let mime_type = captures.name("mime").unwrap().as_str();
                let base64_data = captures.name("data").unwrap().as_str();

                // Decode the base64 data to bytes
                let image_bytes =
                    general_purpose::STANDARD.decode(base64_data)?;

                // Determine the file extension based on MIME type
                let extension = match mime_type {
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    _ => "bin",
                };

                let filename =
                    format!("{}/image_{}.{}", save_dir, index, extension);
                let mut file =
                    File::create(&filename).await.with_context(|| {
                        format!("Error creating file: {}", filename)
                    })?;
                file.write_all(&image_bytes).await.with_context(|| {
                    format!("Error writing to file: {}", filename)
                })?;

                // Like not for if we are generating a music video
                // We need this to only happen sometime
                let filename = format!("./tmp/dalle-1.png");
                let mut file =
                    File::create(&filename).await.with_context(|| {
                        format!("Error creating file: {}", filename)
                    })?;
                file.write_all(&image_bytes).await.with_context(|| {
                    format!("Error writing to file: {}", filename)
                })?;

                println!("Saved image to {}", filename);
            } else {
                eprintln!("Invalid data URL for image at index {}", index);
            }
        }
        Ok(())
    }

    /// Processes images specifically for a music video and saves them with indexing.
    async fn process_images_for_music_video(
        &self,
        raw_json: &[u8],
        save_dir: &str,
        index: usize,
    ) -> Result<()> {
        // Deserialize the JSON response into FalData struct
        let data: FalData = serde_json::from_slice(raw_json)?;

        // Regex to match data URLs in the image URLs
        let data_url_regex =
            Regex::new(r"data:(?P<mime>[\w/]+);base64,(?P<data>.+)")?;

        // Ensure the save directory exists
        create_dir_all(save_dir).await?;

        for (i, image) in data.images.iter().enumerate() {
            if let Some(captures) = data_url_regex.captures(&image.url) {
                let mime_type = captures.name("mime").unwrap().as_str();
                let base64_data = captures.name("data").unwrap().as_str();

                // Decode the base64 data to bytes
                let image_bytes =
                    general_purpose::STANDARD.decode(base64_data)?;

                // Determine the file extension based on MIME type
                let extension = match mime_type {
                    "image/png" => "png",
                    "image/jpeg" => "jpg",
                    _ => "bin",
                };

                // Use the provided index in the filename
                let filename =
                    format!("{}/image_{}_{}.{}", save_dir, index, i, extension);

                // Save the image bytes to a file
                let mut file =
                    File::create(&filename).await.with_context(|| {
                        format!("Error creating file: {}", filename)
                    })?;
                file.write_all(&image_bytes).await.with_context(|| {
                    format!("Error writing to file: {}", filename)
                })?;
                println!("Saved image to {}", filename);
            } else {
                eprintln!("Invalid data URL for image at index {}", i);
            }
        }
        Ok(())
    }

    /// Saves video bytes to the specified file path.
    async fn save_video_bytes(
        &self,
        video_bytes: &[u8],
        filename: &str,
    ) -> Result<()> {
        // Ensure the directory exists
        let dir = std::path::Path::new(filename).parent().unwrap();
        create_dir_all(dir).await?;

        // Write the video data to the file
        tokio::fs::write(&filename, video_bytes)
            .await
            .with_context(|| {
                format!("Failed to write video to {}", filename)
            })?;

        println!("Video saved to: {}", filename);
        Ok(())
    }

    /// Creates an image using the specified model, prompt, and image size, and saves it to the specified directory.
    pub async fn create_image(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
    ) -> Result<()> {
        // Prepare the parameters
        let parameters = serde_json::json!({
            "prompt": prompt,
            "image_size": image_size,
        });

        // Run the model and get the raw JSON response
        let raw_json =
            self.run_model_and_get_raw_json(model, parameters).await?;

        // Save the raw JSON response to a file
        let timestamp = Utc::now().timestamp();
        let json_save_path = format!("{}/{}.json", save_dir, timestamp);
        self.save_raw_json_response(&raw_json, &json_save_path)
            .await?;

        // Process images from the JSON response and save them
        self.process_images_from_json(&raw_json, save_dir).await?;

        Ok(())
    }

    /// Creates an image for a music video using the specified model, prompt, and index, and saves it to the specified directory.
    pub async fn create_image_for_music_video(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
        index: usize,
    ) -> Result<()> {
        // Prepare the parameters
        let parameters = serde_json::json!({
            "prompt": prompt,
            "image_size": image_size,
        });

        // Run the model and get the raw JSON response
        let raw_json =
            self.run_model_and_get_raw_json(model, parameters).await?;

        // Save the raw JSON response to a file
        let timestamp = Utc::now().timestamp();
        let json_save_path = format!("{}/{}.json", save_dir, timestamp);
        self.save_raw_json_response(&raw_json, &json_save_path)
            .await?;

        // Process images specifically for the music video
        self.process_images_for_music_video(&raw_json, save_dir, index)
            .await?;

        Ok(())
    }

    /// Creates a video from the given image file path and saves it to the specified directory.
    pub async fn create_video_from_image(
        &self,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<()> {
        // Encode the image file as a data URI
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let model = "fal-ai/stable-video";

        // Prepare the parameters
        let parameters = serde_json::json!({ "image_url": image_data_uri });

        // Run the model and get the JSON response
        let json = self.run_model_and_get_json(model, parameters).await?;

        // Extract the video URL from the JSON response
        let video_url = json["video"]["url"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract video URL from JSON"))?;

        // Download the video bytes from the URL
        let video_bytes = subd_image_utils::download_video(video_url).await?;

        // Save the video bytes to a file
        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.mp4", save_dir, timestamp);
        self.save_video_bytes(&video_bytes, &filename).await?;

        Ok(())
    }

    /// Submits a request to the Sadtalker model with the given source image and driven audio data URIs.
    pub async fn submit_sadtalker_request(
        &self,
        source_image_data_uri: &str,
        driven_audio_data_uri: &str,
    ) -> Result<String> {
        let model = "fal-ai/sadtalker";
        // Prepare the parameters
        let parameters = serde_json::json!({
            "source_image_url": source_image_data_uri,
            "driven_audio_url": driven_audio_data_uri,
        });
        // Run the model and get the text response
        self.run_model_and_get_text(model, parameters).await
    }
}

/// Creates an image for a music video using the specified id, prompt, and index.
pub async fn create_image_for_music_video(
    id: &str,
    prompt: &str,
    index: usize,
) -> Result<()> {
    let fal_service = FalService::new();
    let model = "fal-ai/fast-sdxl";
    let save_dir = format!("./tmp/music_videos/{}/", id);
    fal_service
        .create_image_for_music_video(
            model,
            prompt,
            "landscape_16_9",
            &save_dir,
            index,
        )
        .await
}

/// Creates a turbo image using the "fal-ai/fast-turbo-diffusion" model.
pub async fn create_turbo_image(prompt: &str) -> Result<()> {
    let fal_service = FalService::new();
    let model = "fal-ai/fast-turbo-diffusion";
    let save_dir = "./tmp/fal_images";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_dir)
        .await
}

/// Creates a fast SD image using the "fal-ai/fast-sdxl" model.
pub async fn create_fast_sd_image(prompt: &str) -> Result<()> {
    let fal_service = FalService::new();
    let model = "fal-ai/fast-sdxl";
    let save_dir = "./tmp/fal_images";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_dir)
        .await
}

/// Creates an image from a prompt and saves it to the specified folder.
pub async fn create_image_from_prompt_in_folder(
    prompt: &str,
    save_folder: &str,
) -> Result<()> {
    let fal_service = FalService::new();
    let model = "fal-ai/stable-cascade";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_folder)
        .await
}

/// Creates a video from the given image file path.
pub async fn create_video_from_image(image_file_path: &str) -> Result<()> {
    let fal_service = FalService::new();
    let save_dir = subd_types::consts::get_ai_videos_dir();
    fal_service
        .create_video_from_image(image_file_path, &save_dir)
        .await
}

/// Submits a request to the Sadtalker model.
pub async fn fal_submit_sadtalker_request(
    source_image_data_uri: &str,
    driven_audio_data_uri: &str,
) -> Result<String> {
    let fal_service = FalService::new();
    fal_service
        .submit_sadtalker_request(source_image_data_uri, driven_audio_data_uri)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_tag::tag;

    #[tokio::test]
    #[tag(fal)]
    async fn test_create_turbo_image() {
        let prompt = "man raccoon";
        let res = create_turbo_image(prompt).await.unwrap();
        dbg!(res);
        assert!(true);
    }

    // async fn test_create_turbo_image() {
    //     let prompt = "man raccoon";
    //     let res = create_turbo_image(prompt).await.unwrap();
    //     dbg!(res);
    //     assert!(true);
    // }
}
