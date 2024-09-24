use crate::models;
use crate::utils;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use fal_rust::client::{ClientCredentials, FalClient};
use tokio::fs::create_dir_all;

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

    /// Creates an image using the specified model, prompt, and image size, and saves it to the specified directory.
    pub async fn create_image(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
        index: Option<usize>,
        obs_background_image_path: Option<&str>,
    ) -> Result<()> {
        let parameters = serde_json::json!({
            "prompt": prompt,
            "image_size": image_size,
        });

        let raw_json =
            self.run_model_and_get_raw_json(model, parameters).await?;

        let timestamp = Utc::now().timestamp();
        let json_save_path = format!("{}/{}.json", save_dir, timestamp);
        utils::save_raw_bytes(&json_save_path, &raw_json).await?;

        // I think this is the problem
        self.process_images(
            &raw_json,
            save_dir,
            &timestamp.to_string(),
            index,
            obs_background_image_path,
        )
        .await?;

        Ok(())
    }

    /// Creates a video from the given image file path and saves it to the specified directory.
    pub async fn create_video_from_image(
        &self,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<()> {
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let parameters = serde_json::json!({ "image_url": image_data_uri });
        let model = "fal-ai/stable-video";
        let json = self.run_model_and_get_json(model, parameters).await?;

        let video_url = json["video"]["url"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract video URL from JSON"))?;

        let video_bytes = subd_image_utils::download_video(video_url).await?;

        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.mp4", save_dir, timestamp);
        utils::save_raw_bytes(&filename, &video_bytes).await?;

        Ok(())
    }

    /// Submits a request to the Sadtalker model with the given source image and driven audio data URIs.
    pub async fn submit_sadtalker_request(
        &self,
        source_image_data_uri: &str,
        driven_audio_data_uri: &str,
    ) -> Result<String> {
        let model = "fal-ai/sadtalker";
        let parameters = serde_json::json!({
            "source_image_url": source_image_data_uri,
            "driven_audio_url": driven_audio_data_uri,
        });
        self.run_model_and_get_text(model, parameters).await
    }

    // =======================================================================================
    // Private

    /// Processes images from the raw JSON response and saves them to the specified directory.
    async fn process_images(
        &self,
        raw_json: &[u8],
        save_dir: &str,
        name: &str,
        index: Option<usize>,
        extra_save_path: Option<&str>,
    ) -> Result<()> {
        let data: models::FalData = serde_json::from_slice(raw_json)?;

        create_dir_all(save_dir).await?;

        for (i, image) in data.images.iter().enumerate() {
            // This extract image data is the problem
            let (image_bytes, extension) =
                utils::extract_image_data(&image.url)?;

            // do we have to pass in the timestamp
            let filename = match index {
                Some(idx) => {
                    format!("{}/{}-{}-{}.{}", save_dir, name, idx, i, extension)
                }
                None => format!("{}/{}-{}.{}", save_dir, name, i, extension),
            };

            utils::save_raw_bytes(&filename, &image_bytes).await?;

            if let Some(extra_path) = extra_save_path {
                utils::save_raw_bytes(extra_path, &image_bytes).await?;
            }

            println!("Saved image to {}", filename);
        }
        Ok(())
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
}
