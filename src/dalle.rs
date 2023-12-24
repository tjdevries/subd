use crate::images;
use anyhow::Result;
use base64::decode;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
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

// ===============================================

pub enum AiImageRequests {
    DalleRequest,
    StableDiffusionRequest,
}

pub struct DalleRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
}

pub struct StableDiffusionRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
}

pub trait GenerateImage {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>;
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

impl GenerateImage for StableDiffusionRequest {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let url = env::var("STABLE_DIFFUSION_URL")
            .map_err(|_| "STABLE_DIFFUSION_URL environment variable not set")
            .unwrap();

        let client = Client::new();
        let req = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({"prompt": prompt}))
            .send();

        let res = async move {
            // Get ridd of the unwraps
            // Then we need to parse to new structure
            let response = match req.await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error with Stable Diffusion response: {}", e);
                    return "".to_string();
                }
            };

            let image_data = match response.bytes().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error with Stable Diffusion image_data: {}", e);
                    return "".to_string();
                }
            };

            let res: SDResponse = match serde_json::from_slice(&image_data) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error parsing SD response: {}", e);
                    return "".to_string();
                }
            };
            let base64 = &res.data[0].b64_json;
            // We rename it image_data because that is what it was origianlly
            let image_data = match general_purpose::STANDARD.decode(base64) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error base64 decoding SD response: {}", e);
                    return "".to_string();
                }
            };

            // // We need a good name for this
            // let mut file = File::create("durf2.png").expect("Failed to create file");
            // file.write_all(&bytes).expect("Failed to write to file");

            // We aren't currently able to generate more than image
            let index = 1;
            // TODO: move this to a function
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let unique_identifier =
                format!("{}_{}_{}", timestamp, index, self.username);

            //
            match save_folder {
                Some(fld) => {
                    let archive_file = format!(
                        "/home/begin/code/subd/archive/{}/{}.png",
                        fld.clone(),
                        unique_identifier
                    );
                    if let Ok(mut file) = File::create(archive_file.clone()) {
                        if let Err(e) = file.write_all(&image_data) {
                            eprintln!("Error writing to file: {}", e);
                        }
                    };
                }
                None => {}
            }

            // TODO: get rid of this hardcoded path
            let archive_file = format!(
                "/home/begin/code/subd/archive/{}.png",
                unique_identifier
            );
            if let Ok(mut file) = File::create(archive_file.clone()) {
                if let Err(e) = file.write_all(&image_data) {
                    eprintln!("Error writing to file: {}", e);
                }
            };

            if set_as_obs_bg {
                let filename =
                    format!("/home/begin/code/subd/tmp/dalle-{}.png", index);
                if let Ok(mut file) = File::create(filename) {
                    if let Err(e) = file.write_all(&image_data) {
                        eprintln!("Error writing to file: {}", e);
                    }
                };
            }

            archive_file
        };
        Box::pin(res)
    }
}

impl GenerateImage for DalleRequest {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let res = async move {
            let mut archive_file = "".to_string();

            match dalle_request(prompt.clone()).await {
                Ok(response) => 'label: {
                    for (index, download_resp) in
                        response.data.iter().enumerate()
                    {
                        println!("Image URL: {} | ", download_resp.url.clone());

                        let timestamp =
                            Utc::now().format("%Y%m%d%H%M%S").to_string();
                        let unique_identifier = format!(
                            "{}_{}_{}",
                            timestamp, index, self.username
                        );
                        let file = format!(
                            "/home/begin/code/subd/archive/{}.png",
                            unique_identifier
                        );
                        let mut image_data = match images::download_image(
                            download_resp.url.clone(),
                            file.clone(),
                        )
                        .await
                        {
                            Ok(val) => {
                                archive_file = file;
                                val
                            }
                            Err(e) => {
                                eprintln!("\nError downloading image: {}", e);
                                break 'label "".to_string();
                            }
                        };

                        if let Some(fld) = save_folder.as_ref() {
                            let f = format!(
                                "/home/begin/code/subd/archive/{}/{}.png",
                                fld, unique_identifier
                            );
                            let _ = File::create(f.clone())
                                .map(|mut f| f.write_all(&mut image_data));
                        }

                        if set_as_obs_bg {
                            let file = format!(
                                "/home/begin/code/subd/tmp/dalle-{}.png",
                                index + 1
                            );
                            let _ = File::create(file.clone())
                                .map(|mut f| f.write_all(&mut image_data));
                        }

                        let csv_file = OpenOptions::new()
                            .write(true)
                            .append(true)
                            .create(true)
                            .open("output.csv");
                        if let Ok(mut f) = csv_file {
                            let _ =
                                writeln!(f, "{},{}", unique_identifier, prompt);
                        }
                    }
                    archive_file
                }
                Err(e) => {
                    eprintln!("Error With Dalle response: {}", e);
                    "".to_string()
                }
            }
        };

        return Box::pin(res);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
