use crate::image_generation;
use crate::images;
use anyhow::Result;
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

#[derive(Serialize, Deserialize, Debug)]
struct ImageResponse {
    created: Option<i64>,
    // created: i64,
    data: Vec<ImageData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    url: String,
}

// ===========================================

pub struct DalleRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
}

// =========================================

impl image_generation::GenerateImage for DalleRequest {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let res = async move {
            if let Ok(response) = dalle_request(prompt.clone()).await {
                for (index, download_resp) in response.data.iter().enumerate() {
                    match process_dalle_request(
                        prompt.clone(),
                        self.username.clone(),
                        index,
                        download_resp,
                        save_folder.clone(),
                        set_as_obs_bg,
                    )
                    .await
                    {
                        Ok(_) => todo!(),
                        Err(e) => {
                            eprintln!("Error processing Dalle Request: {}", e);
                        }
                    };
                }
            }

            return "".to_string();
        };

        return Box::pin(res);
    }
}

// =========================================

async fn dalle_request(prompt: String) -> Result<ImageResponse, String> {
    let api_key = env::var("OPENAI_API_KEY").map_err(|e| e.to_string())?;

    let client = reqwest::Client::new();

    // TODO: read from the database
    let size = "1024x1024";
    // let size = "1280x720";
    // 1280 pixels wide by 720
    let model = "dall-e-3";

    println!("\n\tCalling to DAlle w/ {}", prompt.clone());
    let req = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "prompt": prompt,
            "n": 1,
            "model": model,
            "size": size,
        }))
        .send();

    let response = req.await.map_err(|e| e.to_string())?;

    let dalle_response_text =
        response.text().await.map_err(|e| e.to_string())?;

    let image_response: Result<ImageResponse, String> =
        serde_json::from_str(&dalle_response_text).map_err(|e| e.to_string());
    image_response
}

async fn process_dalle_request(
    prompt: String,
    username: String,
    index: usize,
    download_resp: &ImageData,
    save_folder: Option<String>,
    set_as_obs_bg: bool,
) -> Result<String, String> {
    println!(
        "Processing Dalle Request:: {} | ",
        download_resp.url.clone()
    );

    let (file_as_string, unique_identifier) =
        image_generation::unique_archive_filepath(index, username)
            .map_err(|e| e.to_string())?;

    let f = file_as_string
        .to_str()
        .ok_or("error converting archive path to str")?;

    let mut image_data =
        match images::download_image(download_resp.url.clone(), f.to_string())
            .await
        {
            Ok(val) => val,
            Err(e) => {
                eprintln!("\nError downloading image: {}", e);
                return Err(e.to_string());
            }
        };

    if let Some(fld) = save_folder.clone().as_ref() {
        let f = format!("./subd/archive/{}/{}.png", fld, unique_identifier);
        let filepath = Path::new(&f);
        let pathbuf = PathBuf::from(filepath);
        let file = fs::canonicalize(pathbuf).map_err(|e| e.to_string())?;
        let _ = File::create(file).map(|mut f| f.write_all(&mut image_data));
    }

    if set_as_obs_bg {
        let filename = format!("./tmp/dalle-{}.png", index + 1);
        let filepath = Path::new(&filename);
        let pathbuf = PathBuf::from(filepath);
        if let Ok(file) = fs::canonicalize(pathbuf) {
            let _ = File::create(file.clone())
                .map(|mut f| f.write_all(&mut image_data));
        };
    }

    // Why is this saving to CSV
    let csv_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("output.csv");
    if let Ok(mut f) = csv_file {
        let _ = writeln!(f, "{},{}", unique_identifier, prompt);
    }
    Ok("".to_string())
}
