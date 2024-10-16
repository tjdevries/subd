use anyhow::anyhow;
use anyhow::{Context, Result};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub mod models;
pub mod service;
pub mod utils;

// 3 Types of Requests:
//   Give me an image based on this prompt
//
//   Give me an image based on this URL that points to an image AND this prompt
//
//   Give me an image based on this image data AND a prompt
pub async fn stable_diffusion_from_image(
    request: &models::GenerateAndArchiveRequest,
) -> Result<String> {
    let image_data = service::run_stable_diffusion(request).await?;
    process_stable_diffusion(image_data, request).await
}

pub async fn run_from_prompt(
    request: &models::GenerateAndArchiveRequest,
) -> Result<String> {
    let image_data = service::run_stable_diffusion_from_prompt(request).await?;
    process_stable_diffusion(image_data, request).await
}

// This handles saving the file
// ==============================================================

async fn process_stable_diffusion(
    image_data: Vec<u8>,
    request: &models::GenerateAndArchiveRequest,
) -> Result<String> {
    let archive_file = format!("./archive/{}.png", request.unique_identifier);
    let _ = File::create(Path::new(&archive_file))
        .map(|mut f| f.write_all(&image_data))
        .with_context(|| format!("Error creating: {}", archive_file))?;

    if let Some(fld) = &request.additional_archive_dir {
        let extra_archive_file = format!(
            "./archive/{}/{}.png",
            fld.clone(),
            request.unique_identifier
        );
        let _ = File::create(Path::new(&extra_archive_file))
            .map(|mut f| f.write_all(&image_data))
            .with_context(|| {
                format!("Error creating: {}", extra_archive_file)
            })?;
    }

    if request.set_as_obs_bg {
        let filename = "./tmp/dalle-1.png".to_string();
        let _ = File::create(Path::new(&filename))
            .map(|mut f| f.write_all(&image_data))
            .with_context(|| format!("Error creating: {}", filename))?;
    }

    let string_path = fs::canonicalize(archive_file)?
        .as_os_str()
        .to_str()
        .ok_or(anyhow!("Error converting archive_file to str"))?
        .to_string();
    Ok(string_path)
}
