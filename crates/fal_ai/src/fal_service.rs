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

impl Default for FalService {
    fn default() -> Self {
        Self::new()
    }
}

impl FalService {
    pub fn new() -> Self {
        Self {
            client: FalClient::new(ClientCredentials::from_env()),
        }
    }

    pub async fn create_images_from_model_and_save(
        &self,
        model: &str,
        prompt: &str,
        image_size: &str,
        save_dir: &str,
        obs_background_image_path: Option<&str>,
        filename: Option<&str>,
    ) -> Result<Vec<String>> {
        let parameters = serde_json::json!({
            "prompt": prompt,
            "image_size": image_size,
        });
        let raw_json = self.run_model(model, parameters).await?;

        // Use the passed in filename
        // or use a timestamp as the file name
        let filename = match filename {
            Some(f) => f.to_string(),
            None => {
                format!("{}", Utc::now().timestamp())
            }
        };

        // Use the filename and save_dir to determine
        // where to save the JSON
        let json_save_path = format!("{}/{}.json", save_dir, filename);
        println!("Saving JSON to: {}", json_save_path);

        // Save the JSON response
        self.save_raw_json(&json_save_path, &raw_json).await?;

        // Process the JSON and extract out file_responses
        let file_responses = self
            .parse_json_and_download_images(&raw_json, save_dir, &filename)
            .await?;

        // TODO: Consider improving this, since we are only handling the first file
        // This saves the image downloaded from previous call
        // to an extra path to be referenced by OBS
        if let Some(extra_path) = obs_background_image_path {
            println!("Saving Extra Image to: {}", extra_path);
            self.save_raw_bytes(extra_path, &file_responses[0].image_bytes)
                .await?;
        }

        // Collect all the images paths and return
        let files = file_responses.into_iter().map(|m| m.image_path).collect();

        Ok(files)
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
        let parameters = serde_json::json!({
            "image_url": image_data_uri,
            "prompt": prompt,
        });

        println!("\tAttempting to Generate Video w/ Runway");
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

    pub async fn create_image_from_image(
        &self,
        prompt: &str,
        image_file_path: &str,
        save_dir: &str,
    ) -> Result<String> {
        println!("CREATE IMAGE FROM IMAGE");
        let model = "fal-ai/flux/dev/image-to-image";
        println!("Encoding image_file_path: {}", image_file_path);
        let image_data_uri =
            subd_image_utils::encode_file_as_data_uri(image_file_path).await?;

        // 0.95 is the default
        // more strength, closer to the original image
        let parameters = serde_json::json!({
            "image_url": image_data_uri,
            "prompt": prompt,
            "strength": 0.89,
        });

        println!("Running model and getting JSON");
        let json = self.run_model_and_get_json(model, parameters).await?;

        println!("Create Image From Image Raw JSON: {:?}", json);

        let image_url = json["images"][0]["url"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to extract image URL from JSON"))?;

        let image_bytes =
            subd_image_utils::download_image_to_vec(image_url, None).await?;

        let timestamp = Utc::now().timestamp();
        let filename = format!("{}/{}.mp4", save_dir, timestamp);
        self.save_raw_bytes(&filename, &image_bytes).await?;

        Ok(filename)
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

    async fn parse_json_and_download_images(
        &self,
        raw_json: &[u8],
        save_dir: &str,
        name: &str,
    ) -> Result<Vec<SavedImageResponse>> {
        let data: models::FalData = serde_json::from_slice(raw_json)
            .context("Failed to parse raw JSON into FalData")?;

        create_dir_all(save_dir).await.with_context(|| {
            format!("Failed to create directory '{}'", save_dir)
        })?;

        let image_responses = stream::iter(data.images.iter().enumerate())
            .then(|(_i, image)| async move {
                let extension = "png";
                let filename = format!("{}/{}.{}", save_dir, name, extension);

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
        println!("Running Model: {}", model);
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

    async fn run_model(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_image_from_image() {
        let fal_service = FalService::new();
        let prompt = "Dark Fantasy Anime";
        // let prompt = "Dark Fantasy";
        // let image_file_path = "./tmp/fal_images/1728970948.png";
        let image_file_path = "/Users/beginbot/code/subd/tmp/cool_pepe.png";
        //let image_file_path = "/Users/beginbot/code/subd/tmp/pepe_wizard.jpg";
        let save_dir = "./tmp";
        fal_service
            .create_image_from_image(prompt, image_file_path, save_dir)
            .await
            .unwrap();
        // assert!(false);
        // create_image_from_image()
        // Ok
    }
}
