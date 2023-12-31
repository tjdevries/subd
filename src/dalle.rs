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
        // let url = env::var("STABLE_DIFFUSION_URL_IMG")
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
                Some(fld) => 'label: {
                    let archive_file = format!(
                        "./archive/{}/{}.png",
                        fld.clone(),
                        unique_identifier
                    );
                    let filepath = Path::new(&archive_file);
                    let pathbuf = PathBuf::from(filepath);
                    let file = match fs::canonicalize(pathbuf) {
                        Ok(f) => f,
                        Err(e) => break 'label (),
                    };
                    if let Ok(mut file) = File::create(file) {
                        if let Err(e) = file.write_all(&image_data) {
                            eprintln!("Error writing to file: {}", e);
                        }
                    };
                }
                None => {}
            }

            // TODO: get rid of this hardcoded path
            let archive_file = format!("./archive/{}.png", unique_identifier);
            let filepath = Path::new(&archive_file);
            let pathbuf = PathBuf::from(filepath);
            let file = match fs::canonicalize(pathbuf) {
                Ok(f) => f,
                Err(e) => {
                    return "".to_string();
                }
            };
            if let Ok(mut file) = File::create(file) {
                if let Err(e) = file.write_all(&image_data) {
                    eprintln!("Error writing to file: {}", e);
                }
            };

            if set_as_obs_bg {
                let filename = format!("./subd/tmp/dalle-{}.png", index);
                let filepath = Path::new(&filename);
                let pathbuf = PathBuf::from(filepath);
                let file = fs::canonicalize(pathbuf);
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

// =========================================

fn unique_archive_filepath(
    index: usize,
    username: String,
) -> Result<(PathBuf, String), anyhow::Error> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}_{}", timestamp, index, username);
    let filename = format!("./archive/{}.png", unique_identifier);
    let filepath = Path::new(&filename);
    let pathbuf = PathBuf::from(filepath);
    fs::canonicalize(pathbuf)
        .map_err(|e| anyhow::Error::msg(e.to_string()))
        .map(|v| (v, unique_identifier))
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
        unique_archive_filepath(index, username).map_err(|e| e.to_string())?;

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

impl GenerateImage for DalleRequest {
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
                    let _ = process_dalle_request(
                        prompt.clone(),
                        self.username.clone(),
                        index,
                        download_resp,
                        save_folder.clone(),
                        set_as_obs_bg,
                    )
                    .await;
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
