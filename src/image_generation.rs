use anyhow::Result;
use chrono::Utc;
use core::pin::Pin;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub trait GenerateImage {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>;
}

pub enum AiImageRequests {
    DalleRequest,
    StableDiffusionRequest,
}

pub fn unique_archive_filepath(
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
