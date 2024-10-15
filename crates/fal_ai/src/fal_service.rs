use crate::{models, utils};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use fal_rust::client::{ClientCredentials, FalClient};
use futures::stream::{self, StreamExt, TryStreamExt};
use serde_json::{json, Value};
use tokio::fs::create_dir_all;

pub struct FalService {
    client: FalClient,
}

struct SavedMediaResponse {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Clone)]
enum MediaType {
    Image,
    Video,
}

impl MediaType {
    fn extension(&self) -> &str {
        match self {
            MediaType::Image => "png",
            MediaType::Video => "mp4",
        }
    }
}

impl FalService {
    pub fn new() -> Self {
        Self {
            client: FalClient::new(ClientCredentials::from_env()),
        }
    }
}

impl Default for FalService {
    fn default() -> Self {
        Self::new()
    }
}

impl FalService {
    pub async fn create_images_from_model_and_save(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
        obs_background_image_path: Option<&str>,
        filename: Option<&str>,
    ) -> Result<Vec<String>> {
        let parameters = json!({ "prompt": prompt, "image_size": image_size });

        let (media_responses, json_response) = self
            .generate_media(
                model,
                parameters,
                save_dir,
                filename,
                MediaType::Image,
            )
            .await?;

        let base_filename = filename
            .map(ToString::to_string)
            .unwrap_or_else(|| Utc::now().timestamp().to_string());
        let json_save_path = format!("{}/{}.json", save_dir, base_filename);
        println!("Saving JSON to: {}", json_save_path);

        let raw_json = serde_json::to_vec(&json_response)?;
        self.save_bytes(&json_save_path, &raw_json).await?;

        if let Some(extra_path) = obs_background_image_path {
            println!("Saving Extra Image to: {}", extra_path);
            self.save_bytes(extra_path, &media_responses[0].bytes)
                .await?;
        }

        Ok(media_responses.into_iter().map(|m| m.path).collect())
    }

    pub async fn create_image_from_image(
        &self,
        prompt: &str,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<String> {
        let model = "fal-ai/flux/dev/image-to-image";
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let parameters = json!({
            "image_url": image_data_uri,
            "prompt": prompt,
            "strength": 0.89,
        });

        let (media_responses, _) = self
            .generate_media(model, parameters, save_dir, None, MediaType::Image)
            .await?;

        Ok(media_responses.into_iter().next().unwrap().path)
    }

    pub async fn create_video_from_image_old(
        &self,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<String> {
        let model = "fal-ai/stable-video";
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let parameters = json!({ "image_url": image_data_uri });

        let (media_responses, _) = self
            .generate_media(model, parameters, save_dir, None, MediaType::Video)
            .await?;

        Ok(media_responses.into_iter().next().unwrap().path)
    }

    pub async fn create_video_from_image(
        &self,
        prompt: Option<&str>,
        image_file_path: &str,
        save_dir: &str,
        model: &str,
    ) -> Result<String> {
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;
        let mut parameters = json!({ "image_url": image_data_uri });

        if let Some(prompt) = prompt {
            parameters["prompt"] = json!(prompt);
        }

        let (media_responses, _) = self
            .generate_media(model, parameters, save_dir, None, MediaType::Video)
            .await?;

        Ok(media_responses.into_iter().next().unwrap().path)
    }

    pub async fn submit_sadtalker_request(
        &self,
        source_image_data_uri: &str,
        driven_audio_data_uri: &str,
    ) -> Result<String> {
        let model = "fal-ai/sadtalker";
        let parameters = json!({
            "source_image_url": source_image_data_uri,
            "driven_audio_url": driven_audio_data_uri,
        });
        self.run_model_and_get_text(model, parameters).await
    }

    async fn generate_media(
        &self,
        model: &str,
        parameters: Value,
        save_dir: &str,
        filename: Option<&str>,
        media_type: MediaType,
    ) -> Result<(Vec<SavedMediaResponse>, Value)> {
        let json_response =
            self.run_model_and_get_json(model, parameters).await?;
        let media_urls =
            self.extract_media_urls(&json_response, media_type.clone())?;

        create_dir_all(save_dir).await.with_context(|| {
            format!("Failed to create directory '{}'", save_dir)
        })?;

        let base_filename = filename
            .map(|s| s.to_string())
            .unwrap_or_else(|| Utc::now().timestamp().to_string());

        let saved_media = stream::iter(media_urls.into_iter().enumerate())
            .then(move |(i, url)| {
                let media_type = media_type.clone();
                let save_path = format!(
                    "{}/{}_{}.{}",
                    save_dir,
                    base_filename,
                    i,
                    media_type.extension()
                );
                async move {
                    let media_bytes = self
                        .download_and_save_media(&url, &save_path, media_type)
                        .await?;
                    Ok::<SavedMediaResponse, anyhow::Error>(
                        SavedMediaResponse {
                            path: save_path,
                            bytes: media_bytes,
                        },
                    )
                }
            })
            .try_collect::<Vec<SavedMediaResponse>>()
            .await?;

        Ok((saved_media, json_response))
    }

    pub async fn create_runway_video_from_image(
        &self,
        prompt: &str,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<String> {
        let model = "fal-ai/runway-gen3/turbo/image-to-video";
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        let parameters = json!({
            "image_url": image_data_uri,
            "prompt": prompt,
        });

        let (media_responses, _) = self
            .generate_media(model, parameters, save_dir, None, MediaType::Video)
            .await?;

        Ok(media_responses.into_iter().next().unwrap().path)
    }

    fn extract_media_urls(
        &self,
        json_response: &Value,
        media_type: MediaType,
    ) -> Result<Vec<String>> {
        match media_type {
            MediaType::Image => {
                let images =
                    json_response["images"].as_array().ok_or_else(|| {
                        anyhow!("Failed to parse images array from JSON")
                    })?;
                images
                    .iter()
                    .map(|image| {
                        image["url"].as_str().map(|s| s.to_string()).ok_or_else(
                            || anyhow!("Failed to extract image URL from JSON"),
                        )
                    })
                    .collect()
            }
            MediaType::Video => {
                let video_url = json_response["video"]["url"]
                    .as_str()
                    .ok_or_else(|| {
                        anyhow!("Failed to extract video URL from JSON")
                    })?
                    .to_string();
                Ok(vec![video_url])
            }
        }
    }

    async fn download_and_save_media(
        &self,
        url: &str,
        save_path: &str,
        media_type: MediaType,
    ) -> Result<Vec<u8>> {
        let media_bytes = match media_type {
            MediaType::Image => subd_image_utils::get_image_bytes(url).await?,
            MediaType::Video => {
                subd_image_utils::download_video(url).await?.to_vec()
            }
        };

        self.save_bytes(save_path, &media_bytes).await?;
        println!("Saved media to {}", save_path);
        Ok(media_bytes)
    }

    async fn save_bytes(&self, path: &str, bytes: &[u8]) -> Result<()> {
        utils::save_raw_bytes(path, bytes)
            .await
            .with_context(|| format!("Failed to save bytes to '{}'", path))
    }

    async fn run_model_and_get_json(
        &self,
        model: &str,
        parameters: Value,
    ) -> Result<Value> {
        println!("Running Model: {}", model);
        let response = self.run_model(model, parameters).await?;
        serde_json::from_slice(&response)
            .context("Failed to parse response body into JSON")
    }

    async fn run_model_and_get_text(
        &self,
        model: &str,
        parameters: Value,
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

    async fn run_model(
        &self,
        model: &str,
        parameters: Value,
    ) -> Result<bytes::Bytes> {
        self.client
            .run(model, parameters)
            .await
            .map_err(|e| anyhow!("Failed to run model '{}': {:?}", model, e))?
            .bytes()
            .await
            .with_context(|| {
                format!("Failed to get bytes from model '{}'", model)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_image_from_image() {
        let fal_service = FalService::new();
        let prompt = "Dark Fantasy Anime";
        let image_file_path = "/Users/beginbot/code/subd/tmp/cool_pepe.png";
        let save_dir = "./tmp";
        fal_service
            .create_image_from_image(prompt, image_file_path, save_dir)
            .await
            .unwrap();
    }
}
