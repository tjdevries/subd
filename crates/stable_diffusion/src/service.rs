use crate::models;
use crate::utils;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest;
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Client;
use serde_json::json;
use std::env;

enum RequestDataType {
    Prompt(String),
    Img(Form),
}

pub async fn run_stable_diffusion(
    request: &models::GenerateAndArchiveRequest,
) -> Result<Vec<u8>> {
    let form = form_builder(request)?;
    let request = RequestDataType::Img(form);
    Ok(call_prompt_api(request).await?)
}

fn form_builder(request: &models::GenerateAndArchiveRequest) -> Result<Form> {
    let default_strength = 0.4;
    let form = reqwest::multipart::Form::new()
        .text(
            "strength",
            format!("{}", request.strength.unwrap_or(default_strength)),
        )
        .text("prompt", request.prompt.clone());

    let form = match &request.request_type {
        models::RequestType::Img2ImgFile(filename) => {
            let (path, buffer) = utils::resize_image(
                request.unique_identifier.clone(),
                filename.clone(),
            )?;

            let p = Part::bytes(buffer)
                .mime_str("image/png")?
                .file_name(path.clone());
            form.part("file", p)
        }
        models::RequestType::Img2ImgURL(url) => {
            form.text("image_url", url.clone())
        }

        // we can't handle the prompt with our current setup
        models::RequestType::Prompt2Img => {
            return Err(anyhow!("Img2Img not implemented"));
        }
    };

    Ok(form)
}

async fn call_prompt_api(request_type: RequestDataType) -> Result<Vec<u8>> {
    let req = match request_type {
        RequestDataType::Prompt(prompt) => {
            let url = env::var("STABLE_DIFFUSION_URL")?;
            Client::new()
                .post(url)
                .header("Content-Type", "application/json")
                .json(&json!({"prompt": prompt}))
        }
        RequestDataType::Img(form) => {
            let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
            Client::new().post(url).multipart(form)
        }
    };

    let res = req
        .send()
        .await?
        .bytes()
        .await
        .map(|i| serde_json::from_slice::<models::SDResponse>(&i))
        .with_context(|| {
            "Couldn't parse Stable Diffusion response into SDResponse"
        })??;

    let base64 = &res.data[0].b64_json;
    general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| anyhow!(e.to_string()))
        .and_then(|v| Ok(v))
}
