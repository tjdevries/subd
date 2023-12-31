use crate::image_generation;
use crate::images;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::decode;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use core::pin::Pin;
use reqwest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub struct StableDiffusionRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SDResponse {
    data: Vec<SDResponseData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SDResponseData {
    b64_json: String,
    revised_prompt: String,
}

async fn generate_and_download_image(
    prompt: String,
    save_folder: Option<String>,
    set_as_obs_bg: bool,
) -> Result<()> {
    let username = "beginbot".to_string();
    let index = 1;
    let image_data = match download_stable_diffusion(prompt).await {
        Ok(i) => i,
        Err(e) => {
            println!("Error downloading stable diffusion: {}", e);
            return Err(anyhow!("Error downloading stable diffusion"));
        }
    };

    match process_stable_diffusion(
        username,
        index,
        image_data.clone().into(),
        save_folder,
        set_as_obs_bg,
    )
    .await
    {
        Ok(_) => println!("Successfully processed stable diffusion request"),
        Err(e) => println!("Error processing stable diffusion request: {}", e),
    }

    // If it fails at this point, whoops, not much we can do
    Ok(())
}

impl image_generation::GenerateImage for StableDiffusionRequest {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let res = async move {
            let _ =
                generate_and_download_image(prompt, save_folder, set_as_obs_bg)
                    .await;

            // We do this because of async traits, they ruined our life
            // Why do we need to return a string
            "".to_string()
        };

        Box::pin(res)
    }
}

async fn process_stable_diffusion(
    username: String,
    index: usize,
    image_data: Vec<u8>,
    save_folder: Option<String>,
    set_as_obs_bg: bool,
) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}_{}", timestamp, index, username);

    println!("Creating Original Archive Image");
    // This saves the original archive of the image
    let archive_file = format!("./archive/{}.png", unique_identifier);
    let _ = File::create(&Path::new(&archive_file))
        .map(|mut f| f.write_all(&image_data))
        .with_context(|| format!("Error creating: {}", archive_file))?;

    println!("Creating Extra Archive Image");
    // This saves an extra copy of the image, into a folder
    if let Some(fld) = save_folder {
        let extra_archive_file =
            format!("./archive/{}/{}.png", fld.clone(), unique_identifier);
        let _ = File::create(&Path::new(&extra_archive_file))
            .map(|mut f| f.write_all(&image_data))
            .with_context(|| {
                format!("Error creating: {}", extra_archive_file)
            })?;
    }

    println!("Set OBS BG");
    // If we write to this file, it's the background of OBS
    if set_as_obs_bg {
        let filename = format!("./tmp/dalle-{}.png", index);
        let _ = File::create(&Path::new(&filename))
            .map(|mut f| f.write_all(&image_data))
            .with_context(|| format!("Error creating: {}", filename))?;
    }

    Ok(())
}

async fn download_stable_diffusion(prompt: String) -> Result<Vec<u8>> {
    // let url = env::var("STABLE_DIFFUSION_URL_IMG")
    let url = env::var("STABLE_DIFFUSION_URL")?;

    let client = Client::new();
    let req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json!({"prompt": prompt}))
        .send();

    // How would we use and_then
    let res = req
        .await?
        .bytes()
        .await
        .map(|i| serde_json::from_slice::<SDResponse>(&i))??;

    let base64 = &res.data[0].b64_json;

    general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_generation::GenerateImage;

    #[tokio::test]
    async fn test_stable_d() -> Result<()> {
        let prompt = "beach sunset beautiful dank af frog time".to_string();
        let username = "beginbot".to_string();
        let req = StableDiffusionRequest {
            prompt: prompt.clone(),
            username,
            amount: -2,
        };

        let image_data = download_stable_diffusion(req.prompt.clone()).await?;

        let _ = process_stable_diffusion(
            "beginbot".to_string(),
            1,
            image_data,
            None,
            false,
        )
        .await;
        Ok(())
    }

    #[test]
    fn test_parsing_carls_images() {
        let filepath = "./archive/b.json";
        let srcdir = PathBuf::from(filepath);
        let f = fs::canonicalize(srcdir).unwrap();

        let mut file = File::open(f).unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let res: SDResponse = serde_json::from_str(&contents).unwrap();
        let base64 = &res.data[0].b64_json;
        let bytes = general_purpose::STANDARD.decode(base64).unwrap();

        // We need a good name for this
        let mut file =
            File::create("durf2.png").expect("Failed to create file");
        file.write_all(&bytes).expect("Failed to write to file");
        //
        // // Unless it's none
        // let _content = &res.choices[0].message.content;

        // assert_eq!(srcdir.to_string(), "".to_string());
    }
}
