use crate::request;
use crate::response;
use crate::utils;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest;
use reqwest::blocking::multipart;
use reqwest::multipart::Part;
use reqwest::Client;
use serde_json::json;
use std::env;
use std::io::Read;

// Do we want to resize here
pub async fn download_stable_diffusion_img2img(
    prompt: String,
    unique_identifier: String,
    strength: Option<f32>,
    request_type: request::Img2ImgRequestType,
) -> Result<Vec<u8>> {
    let default_strength = 0.6;
    let strength = strength.map_or(default_strength, |s| {
        if s > 0.1 && s < 1.0 {
            s
        } else {
            default_strength
        }
    });

    let form = reqwest::multipart::Form::new()
        .text("strength", format!("{}", strength))
        .text("prompt", prompt);

    let form = match request_type {
        request::Img2ImgRequestType::Image(filename) => {
            let (path, buffer) = utils::resize_image(
                unique_identifier.clone(),
                filename.clone(),
            )?;

            let p = Part::bytes(buffer)
                .mime_str("image/png")?
                .file_name(path.clone());
            form.part("file", p)
        }
        request::Img2ImgRequestType::Url(url) => form.text("image_url", url),
    };

    // Call and parse stable
    let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
    let client = Client::new();
    let res = client
        .post(url)
        .multipart(form)
        .send()
        .await?
        .bytes()
        .await?;

    let b = serde_json::from_slice::<response::SDResponse>(&res)
        .with_context(|| format!("Erroring Parsing Dalle img2img"))?;

    let base64 = &b.data[0].b64_json;

    general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| anyhow!(e.to_string()))
        .and_then(|v| Ok(v))
}

pub async fn download_stable_diffusion(prompt: String) -> Result<Vec<u8>> {
    let url = env::var("STABLE_DIFFUSION_URL")?;
    let client = Client::new();
    let req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json!({"prompt": prompt}))
        .send();

    let res = req
        .await?
        .bytes()
        .await
        .map(|i| serde_json::from_slice::<response::SDResponse>(&i))
        .with_context(|| {
            "Couldn't parse Stable Diffusion response into SDResponse"
        })??;

    let base64 = &res.data[0].b64_json;

    general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_stable_d() -> Result<()> {
        let prompt = "batman".to_string();
        let username = "beginbot".to_string();
        let req = request::StableDiffusionRequest {
            prompt: prompt.clone(),
            username: username.clone(),
            amount: -2,
        };

        let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
        let filename = "".to_string();
        let unique_identifier = "".to_string();
        let image_data = download_stable_diffusion_img2img(
            req.prompt.clone(),
            unique_identifier,
            None,
            request::Img2ImgRequestType::Image(filename),
        )
        .await?;
        Ok(())

        // This needs to be moved back to libs
        // let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        // let unique_identifier = format!("{}_{}", timestamp, username);
        // let _ = process_stable_diffusion(
        //     unique_identifier,
        //     image_data,
        //     None,
        //     false,
        // )
        // .await;
        // Ok(())
    }
}
