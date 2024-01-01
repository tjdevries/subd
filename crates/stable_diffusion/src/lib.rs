use anyhow::anyhow;
use anyhow::{Context, Result};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
mod models;
mod service;
mod utils;

pub enum RequestType {
    Img2ImgFile(String),
    Img2ImgURL(String),
    Prompt2Img(),
}

// Filename
// Unique Identifier
struct GenerateAndArchiveRequest {
    prompt: String,
    unique_identifier: String,
    request_type: RequestType,
    set_as_obs_bg: bool,
    additional_archive_dir: Option<String>,
    strength: Option<f32>,
}

// 3 Types of Requests:
//   Give me an image based on this prompt
//
//   Give me an image based on this URL that points to an image AND this prompt
//
//   Give me an image based on this image data AND a prompt

pub async fn stable_diffusion_from_image(
    prompt: String,
    request_type: RequestType,
    unique_identifier: String,
    set_as_obs_bg: bool,
    additional_archive_dir: Option<String>,
    strength: Option<f32>,
) -> Result<String> {
    let image_data = service::download_stable_diffusion_img2img(
        prompt,
        unique_identifier.clone(),
        strength,
        request_type,
    )
    .await?;

    process_stable_diffusion(
        unique_identifier,
        image_data.clone().into(),
        additional_archive_dir,
        set_as_obs_bg,
    )
    .await
}

pub async fn stable_diffusion_from_prompt(
    prompt: String,
    filename: String,
    set_as_obs_bg: bool,
    additional_archive_dir: Option<String>,
) -> Result<()> {
    let image_data = service::download_stable_diffusion(prompt)
        .await
        .with_context(|| "Error downloading stable diffusion")?;

    match process_stable_diffusion(
        filename,
        image_data.clone().into(),
        additional_archive_dir,
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

// This handles saving the file
// ==============================================================

async fn process_stable_diffusion(
    unique_identifier: String,
    image_data: Vec<u8>,
    save_folder: Option<String>,
    set_as_obs_bg: bool,
) -> Result<String> {
    let archive_file = format!("./archive/{}.png", unique_identifier);
    let _ = File::create(&Path::new(&archive_file))
        .map(|mut f| f.write_all(&image_data))
        .with_context(|| format!("Error creating: {}", archive_file))?;

    if let Some(fld) = save_folder {
        let extra_archive_file =
            format!("./archive/{}/{}.png", fld.clone(), unique_identifier);
        let _ = File::create(&Path::new(&extra_archive_file))
            .map(|mut f| f.write_all(&image_data))
            .with_context(|| {
                format!("Error creating: {}", extra_archive_file)
            })?;
    }

    if set_as_obs_bg {
        let filename = format!("./tmp/dalle-1.png");
        let _ = File::create(&Path::new(&filename))
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
