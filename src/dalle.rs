use anyhow::Result;
use chrono::Utc;
use core::pin::Pin;
use reqwest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
struct ImageResponse {
    created: i64,
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

impl GenerateImage for AiImageRequests {
    fn generate_image(
        &self,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let res = async move { "".to_string() };
        Box::pin(res)
    }
}

// pub struct DalleRequest  = AiImageRequest;
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
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>;
}

impl GenerateImage for StableDiffusionRequest {
    fn generate_image(
        &self,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let url = env::var("STABLE_DIFFUSION_URL")
            .map_err(|_| "STABLE_DIFFUSION_URL environment variable not set")
            .unwrap();

        let client = Client::new();
        let req = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({"prompt": self.prompt}))
            .send();

        let res = async move {
            let response = req.await.unwrap();

            let image_data = response.bytes().await.unwrap();

            // We aren't currently able to generate more than image
            let index = 1;
            // TODO: move this to a function
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let unique_identifier =
                format!("{}_{}_{}", timestamp, index, self.username);
            let archive_file = format!("./archive/{}.png", unique_identifier);

            let mut file = File::create(archive_file.clone()).unwrap();
            file.write_all(&image_data).unwrap();

            let filename = format!("./tmp/dalle-{}.png", index);
            let mut file = File::create(filename).unwrap();
            file.write_all(&image_data).unwrap();
            archive_file
        };
        Box::pin(res)
    }
}

impl GenerateImage for DalleRequest {
    fn generate_image(
        &self,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
    {
        let api_key = env::var("OPENAI_API_KEY").unwrap();

        // TODO: This was supposed to be for saving to the file
        // which we aren't doing yet
        let _truncated_prompt =
            self.prompt.chars().take(80).collect::<String>();
        let client = reqwest::Client::new();

        // TODO: read from the database
        let size = "1024x1024";
        let model = "dall-e-3";

        let req = client
            .post("https://api.openai.com/v1/images/generations")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "prompt": self.prompt,
                "n": self.amount,
                "model": model,
                "size": size,
            }))
            .send();

        let res = async move {
            let response = req.await.unwrap();

            let dalle_response_text = response.text().await.unwrap();

            let mut csv_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true) // This will create the file if it doesn't exist
                .open("output.csv")
                .unwrap();

            let image_response: Result<ImageResponse, _> =
                serde_json::from_str(&dalle_response_text);

            let mut archive_file = "".to_string();

            match image_response {
                Ok(response) => {
                    for (index, image_data) in response.data.iter().enumerate()
                    {
                        println!("Image URL: {} | ", image_data.url.clone());
                        let image_data = reqwest::get(image_data.url.clone())
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap()
                            .to_vec();

                        // "id": 9612607,
                        // request for AI_image_filename
                        let timestamp =
                            Utc::now().format("%Y%m%d%H%M%S").to_string();
                        let unique_identifier = format!(
                            "{}_{}_{}",
                            timestamp, index, self.username
                        );
                        archive_file = format!(
                            "/home/begin/code/subd/archive/{}.png",
                            unique_identifier
                        );

                        let mut file =
                            File::create(archive_file.clone()).unwrap();
                        file.write_all(&image_data).unwrap();

                        writeln!(
                            csv_file,
                            "{},{}",
                            unique_identifier, self.prompt
                        )
                        .unwrap();

                        // how do you get it to return a full path

                        let filename = format!(
                            "/home/begin/code/subd/tmp/dalle-{}.png",
                            index + 1
                        );
                        let mut file = File::create(filename.clone()).unwrap();
                        file.write_all(&image_data).unwrap();
                    }
                    archive_file
                }
                Err(e) => {
                    eprintln!("Error deserializing response: {}", e);
                    "".to_string()
                }
            }
        };

        return Box::pin(res);
    }
}

// macro_rules! new_request_type {
//     ($name:ident) => {
//         struct $name {
//             field1: i32,
//             field2: String,
//         }
//
//         impl $name {
//             fn new(field1: i32, field2: String) -> Self {
//                 $name { field1, field2 }
//             }
//
//             fn field1(self) -> i32 {
//                 self.field1
//             }
//         }
//     };
// }
//
// new_request_type!(StableDiffusionRequest);
// new_request_type!(AiImageRequest);
//
// fn main() {
//     let a = StableDiffusionRequest::new(10, "Hello".to_string());
//     let b = AiImageRequest::new(20, "World".to_string());
//
//     println!("{}", b.field1());
//
// }

//
//
// pub async fn dalle_time(
//     contents: String,
//     username: String,
//     amount: i32,
// ) -> Result<String, reqwest::Error> {
//     let api_key = env::var("OPENAI_API_KEY").unwrap();
//
//     // TODO: This was supposed to be for saving to the file
//     // which we aren't doing yet
//     let _truncated_prompt = contents.chars().take(80).collect::<String>();
//     let client = reqwest::Client::new();
//
//     let size = "1024x1024";
//     // TODO: read from the database
//     let model = "dall-e-3";
//
//     let response = client
//         .post("https://api.openai.com/v1/images/generations")
//         .header("Content-Type", "application/json")
//         .header("Authorization", format!("Bearer {}", api_key))
//         .json(&serde_json::json!({
//             "prompt": contents,
//             "n": amount,
//             "model": model,
//             "size": size,
//         }))
//         .send()
//         .await?;
//
//     let dalle_response_text = response.text().await?;
//
//     let mut csv_file = OpenOptions::new()
//         .write(true)
//         .append(true)
//         .create(true) // This will create the file if it doesn't exist
//         .open("output.csv")
//         .unwrap();
//
//     let image_response: Result<ImageResponse, _> =
//         serde_json::from_str(&dalle_response_text);
//     match image_response {
//         Ok(response) => {
//             for (index, image_data) in response.data.iter().enumerate() {
//                 println!("Image URL: {} | ", image_data.url.clone());
//                 let image_data = reqwest::get(image_data.url.clone())
//                     .await?
//                     .bytes()
//                     .await?
//                     .to_vec();
//
//                 let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
//                 let unique_identifier =
//                     format!("{}_{}_{}", timestamp, index, username);
//                 let archive_file = format!(
//                     "/home/begin/code/subd/archive/{}.png",
//                     unique_identifier
//                 );
//
//                 let mut file = File::create(archive_file.clone()).unwrap();
//                 file.write_all(&image_data).unwrap();
//
//                 writeln!(csv_file, "{},{}", unique_identifier, contents)
//                     .unwrap();
//
//                 // how do you get it to return a full path
//
//                 let filename = format!(
//                     "/home/begin/code/subd/tmp/dalle-{}.png",
//                     index + 1
//                 );
//                 let mut file = File::create(filename.clone()).unwrap();
//                 file.write_all(&image_data).unwrap();
//
//                 return Ok(archive_file);
//             }
//         }
//         Err(e) => {
//             eprintln!("Error deserializing response: {}", e);
//         }
//     }
//     Ok("".to_string())
// }
