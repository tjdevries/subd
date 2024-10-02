use crate::models;
use crate::utils;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use fal_rust::client::{ClientCredentials, FalClient};
use futures::stream::{self, StreamExt, TryStreamExt};
use tokio::fs::create_dir_all;

pub struct FalService {
    client: FalClient,
}

// We might want to return Vec of theses
struct SavedImageResponse {
    image_path: String,
    image_bytes: Vec<u8>,
}

impl FalService {
    pub fn new() -> Self {
        Self {
            client: FalClient::new(ClientCredentials::from_env()),
        }
    }

    pub async fn create_image(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
        index: Option<usize>,
        obs_background_image_path: Option<&str>,
    ) -> Result<Vec<String>> {
        let parameters = serde_json::json!({
            "prompt": prompt,
            "image_size": image_size,
        });

        let raw_json =
            self.run_model_and_get_raw_json(model, parameters).await?;

        let timestamp = Utc::now().timestamp();
        let json_save_path = format!("{}/{}.json", save_dir, timestamp);

        println!("Saving JSON to: {}", json_save_path);
        self.save_raw_json(&json_save_path, &raw_json).await?;

        let file_responses = self
            .process_images(&raw_json, save_dir, &timestamp.to_string(), index)
            .await?;

        // TODO: Consider improving this
        // while only handle the first file
        if let Some(extra_path) = obs_background_image_path {
            self.save_raw_bytes(extra_path, &file_responses[0].image_bytes)
                .await?;
        }

        let files = file_responses.into_iter().map(|m| m.image_path).collect();

        Ok(files)
    }

    pub async fn create_video_from_image(
        &self,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<String> {
        let model = "fal-ai/stable-video";
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let parameters = serde_json::json!({ "image_url": image_data_uri });
        let json = self.run_model_and_get_json(model, parameters).await?;

        println!("Create Video From Image Raw JSON: {:?}", json);

        let video_url = json["video"]["url"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract video URL from JSON"))?;

        let video_bytes = subd_image_utils::download_video(video_url).await?;

        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.mp4", save_dir, timestamp);
        self.save_raw_bytes(&filename, &video_bytes).await?;

        Ok(filename)
    }

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
    // Private methods

    async fn save_raw_json(&self, path: &str, raw_json: &[u8]) -> Result<()> {
        utils::save_raw_bytes(path, raw_json)
            .await
            .context("Failed to save raw JSON")
    }

    async fn save_raw_bytes(&self, path: &str, bytes: &[u8]) -> Result<()> {
        utils::save_raw_bytes(path, bytes)
            .await
            .with_context(|| format!("Failed to save bytes to '{}'", path))
    }

    async fn process_images(
        &self,
        raw_json: &[u8],
        save_dir: &str,
        name: &str,
        index: Option<usize>,
    ) -> Result<Vec<SavedImageResponse>> {
        let data: models::FalData = serde_json::from_slice(raw_json)
            .context("Failed to parse raw JSON into FalData")?;

        create_dir_all(save_dir).await.with_context(|| {
            format!("Failed to create directory '{}'", save_dir)
        })?;

        let image_responses = stream::iter(data.images.iter().enumerate())
            .then(|(i, image)| async move {
                // Double indexes are dumb here I think
                let filename =
                    self.construct_filename(&save_dir, &name, index, i);
                let image_bytes =
                    self.save_image(&image.url, &filename).await?;
                Ok::<SavedImageResponse, anyhow::Error>(SavedImageResponse {
                    image_path: filename,
                    image_bytes,
                })
            })
            .try_collect::<Vec<SavedImageResponse>>()
            .await?;

        Ok(image_responses)
    }

    fn construct_filename(
        &self,
        save_dir: &str,
        name: &str,
        index: Option<usize>,
        i: usize,
    ) -> String {
        let extension = "png";
        match index {
            Some(idx) => {
                format!("{}/{}-{}-{}.{}", save_dir, name, idx, i, extension)
            }
            None => format!("{}/{}-{}.{}", save_dir, name, i, extension),
        }
    }

    async fn save_image(
        &self,
        image_url: &str,
        save_path: &str,
    ) -> Result<Vec<u8>> {
        let image_bytes = subd_image_utils::get_image_bytes(image_url)
            .await
            .with_context(|| {
                format!("Failed to get image bytes from '{}'", image_url)
            })?;

        self.save_raw_bytes(save_path, &image_bytes).await?;

        println!("Saved image to {}", save_path);
        Ok(image_bytes)
    }

    async fn run_model_and_get_json(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let response =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        let body = response
            .text()
            .await
            .context("Failed to get response text from model")?;

        serde_json::from_str(&body)
            .context("Failed to parse response body into JSON")
    }

    async fn run_model_and_get_raw_json(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<bytes::Bytes> {
        let response =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        response.bytes().await.with_context(|| {
            format!("Failed to get bytes from model '{}'", model)
        })
    }

    async fn run_model_and_get_text(
        &self,
        model: &str,
        parameters: serde_json::Value,
    ) -> Result<String> {
        let response =
            self.client.run(model, parameters).await.map_err(|e| {
                anyhow!("Failed to run model '{}': {:?}", model, e)
            })?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Request to model '{}' failed with status: {}",
                model,
                response.status()
            ));
        }

        response.text().await.with_context(|| {
            format!("Error getting text from model '{}'", model)
        })
    }
}
