use crate::image_generation;
use crate::images;
use crate::obs_source;
use anyhow::anyhow;
use anyhow::{Context, Result};
use base64::decode;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use core::pin::Pin;
use reqwest;
use reqwest::blocking::multipart;
use reqwest::multipart::Part;
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

// pub struct StableDiffusionImg2ImgRequest {
//     pub prompt: String,
//     pub filename: String,
//     pub unique_identifier: String,
// }
//
// pub struct StableDiffusionRequest {
//     pub prompt: String,
//     pub username: String,
//     pub amount: i32,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct SDResponse {
//     data: Vec<SDResponseData>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct SDResponseData {
//     b64_json: String,
//     revised_prompt: String,
// }
//
// pub enum StableDiffusionRequests {
//     StableDiffusionImg2ImgRequest,
//     StableDiffusionRequest,
// }
//
// enum Img2ImgRequestType {
//     Image(String),
//     Url(String),
// }
//
// impl image_generation::GenerateImage for StableDiffusionImg2ImgRequest {
//     fn generate_image(
//         &self,
//         prompt: String,
//         save_folder: Option<String>,
//         set_as_obs_bg: bool,
//     ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
//     {
//         let res = async move {
//             let filename = self.filename.clone();
//             let unique_identifier = self.unique_identifier.clone();
//             let _ = stable_diffusion_from_image(
//                 prompt,
//                 filename,
//                 unique_identifier,
//                 save_folder,
//                 set_as_obs_bg,
//                 None,
//             )
//             .await;
//
//             // We do this because of async traits, they ruined our life
//             // Why do we need to return a string
//             "".to_string()
//         };
//
//         Box::pin(res)
//     }
// }
//
// impl image_generation::GenerateImage for StableDiffusionRequest {
//     fn generate_image(
//         &self,
//         prompt: String,
//         save_folder: Option<String>,
//         set_as_obs_bg: bool,
//     ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>
//     {
//         let username = "beginbot".to_string();
//         let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
//         let unique_identifier = format!("{}_{}", timestamp, username);
//
//         let res = async move {
//             let _ = stable_diffusion_from_prompt(
//                 unique_identifier,
//                 prompt,
//                 save_folder,
//                 set_as_obs_bg,
//             )
//             .await;
//
//             // We do this because of async traits, they ruined our life
//             // Why do we need to return a string
//             "".to_string()
//         };
//
//         Box::pin(res)
//     }
// }
//
// pub async fn stable_diffusion_from_image(
//     prompt: String,
//     filename: String,
//     unique_identifier: String,
//     save_folder: Option<String>,
//     set_as_obs_bg: bool,
//     strength: Option<f32>,
// ) -> Result<String> {
//     let image_data = download_stable_diffusion_img2img(
//         prompt,
//         unique_identifier.clone(),
//         strength,
//         Img2ImgRequestType::Image(filename),
//     )
//     .await?;
//
//     process_stable_diffusion(
//         unique_identifier,
//         image_data.clone().into(),
//         save_folder,
//         set_as_obs_bg,
//     )
//     .await
// }
//
// pub async fn stable_diffusion_from_prompt(
//     prompt: String,
//     filename: String,
//     save_folder: Option<String>,
//     set_as_obs_bg: bool,
// ) -> Result<()> {
//     let url = env::var("STABLE_DIFFUSION_URL")?;
//     let image_data = download_stable_diffusion(prompt, url)
//         .await
//         .with_context(|| "Error downloading stable diffusion")?;
//
//     match process_stable_diffusion(
//         filename,
//         image_data.clone().into(),
//         save_folder,
//         set_as_obs_bg,
//     )
//     .await
//     {
//         Ok(_) => println!("Successfully processed stable diffusion request"),
//         Err(e) => println!("Error processing stable diffusion request: {}", e),
//     }
//
//     // If it fails at this point, whoops, not much we can do
//     Ok(())
// }
//
// async fn download_stable_diffusion_img2img(
//     prompt: String,
//     unique_identifier: String,
//     strength: Option<f32>,
//     request_type: Img2ImgRequestType,
// ) -> Result<Vec<u8>> {
//     let default_strength = 0.6;
//     let strength = strength.map_or(default_strength, |s| {
//         if s > 0.1 && s < 1.0 {
//             s
//         } else {
//             default_strength
//         }
//     });
//
//     let form = reqwest::multipart::Form::new()
//         .text("strength", format!("{}", strength))
//         .text("prompt", prompt);
//
//     let form = match request_type {
//         Img2ImgRequestType::Image(filename) => {
//             let (path, buffer) = images::resize_image(
//                 unique_identifier.clone(),
//                 filename.clone(),
//             )?;
//
//             let p = Part::bytes(buffer)
//                 .mime_str("image/png")?
//                 .file_name(path.clone());
//             form.part("file", p)
//         }
//         Img2ImgRequestType::Url(url) => form.text("image_url", url),
//     };
//
//     // Call and parse stable
//     let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
//     let client = Client::new();
//     let res = client
//         .post(url)
//         .multipart(form)
//         .send()
//         .await?
//         .bytes()
//         .await?;
//
//     let b = serde_json::from_slice::<SDResponse>(&res)
//         .with_context(|| format!("Erroring Parsing Dalle img2img"))?;
//
//     let base64 = &b.data[0].b64_json;
//
//     general_purpose::STANDARD
//         .decode(base64)
//         .map_err(|e| anyhow!(e.to_string()))
//         .and_then(|v| Ok(v))
// }
//
// async fn download_stable_diffusion(
//     prompt: String,
//     url: String,
// ) -> Result<Vec<u8>> {
//     let client = Client::new();
//     let req = client
//         .post(&url)
//         .header("Content-Type", "application/json")
//         .json(&json!({"prompt": prompt}))
//         .send();
//
//     let res = req
//         .await?
//         .bytes()
//         .await
//         .map(|i| serde_json::from_slice::<SDResponse>(&i))
//         .with_context(|| {
//             "Couldn't parse Stable Diffusion response into SDResponse"
//         })??;
//
//     let base64 = &res.data[0].b64_json;
//
//     general_purpose::STANDARD
//         .decode(base64)
//         .map_err(|e| anyhow!(e.to_string()))
// }
//
// async fn process_stable_diffusion(
//     unique_identifier: String,
//     image_data: Vec<u8>,
//     save_folder: Option<String>,
//     set_as_obs_bg: bool,
// ) -> Result<String> {
//     let archive_file = format!("./archive/{}.png", unique_identifier);
//     let _ = File::create(&Path::new(&archive_file))
//         .map(|mut f| f.write_all(&image_data))
//         .with_context(|| format!("Error creating: {}", archive_file))?;
//
//     if let Some(fld) = save_folder {
//         let extra_archive_file =
//             format!("./archive/{}/{}.png", fld.clone(), unique_identifier);
//         let _ = File::create(&Path::new(&extra_archive_file))
//             .map(|mut f| f.write_all(&image_data))
//             .with_context(|| {
//                 format!("Error creating: {}", extra_archive_file)
//             })?;
//     }
//
//     if set_as_obs_bg {
//         let filename = format!("./tmp/dalle-1.png");
//         let _ = File::create(&Path::new(&filename))
//             .map(|mut f| f.write_all(&image_data))
//             .with_context(|| format!("Error creating: {}", filename))?;
//     }
//
//     let string_path = fs::canonicalize(archive_file)?
//         .as_os_str()
//         .to_str()
//         .ok_or(anyhow!("Error converting archive_file to str"))?
//         .to_string();
//     Ok(string_path)
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::image_generation::GenerateImage;
//
//     #[tokio::test]
//     async fn test_stable_d() -> Result<()> {
//         let prompt = "batman".to_string();
//         let username = "beginbot".to_string();
//         let req = StableDiffusionRequest {
//             prompt: prompt.clone(),
//             username,
//             amount: -2,
//         };
//
//         let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
//         let filename = "".to_string();
//         let unique_identifier = "".to_string();
//         let image_data = download_stable_diffusion_img2img(
//             req.prompt.clone(),
//             unique_identifier,
//             None,
//             Img2ImgRequestType::Image(filename),
//         )
//         .await?;
//
//         let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
//         let unique_identifier = format!("{}_{}", timestamp, username);
//         let _ = process_stable_diffusion(
//             unique_identifier,
//             image_data,
//             None,
//             false,
//         )
//         .await;
//         Ok(())
//     }
//
//     #[test]
//     fn test_parsing_carls_images() {
//         let filepath = "./archive/b.json";
//         // let srcdir = PathBuf::from(filepath);
//         let f = fs::canonicalize(filepath).unwrap();
//
//         let mut file = File::open(f).unwrap();
//         let mut contents = String::new();
//         let _ = file.read_to_string(&mut contents);
//
//         let res: SDResponse = serde_json::from_str(&contents).unwrap();
//         let base64 = &res.data[0].b64_json;
//         let bytes = general_purpose::STANDARD.decode(base64).unwrap();
//
//         // We need a good name for this
//         let mut file =
//             File::create("durf2.png").expect("Failed to create file");
//         file.write_all(&bytes).expect("Failed to write to file");
//         //
//         // // Unless it's none
//         // let _content = &res.choices[0].message.content;
//
//         // assert_eq!(srcdir.to_string(), "".to_string());
//     }
// }
