use ai_images;
use ai_images::image_generation::GenerateImage;
use anyhow::anyhow;
use anyhow::Result;
use chrono::{DateTime, Utc};
use obs_service::obs_source;
use obws::requests::custom::source_settings::SlideshowFile;
use obws::Client as OBSClient;
use rodio::*;
use stable_diffusion;
use std::fs;
use std::fs::create_dir;
use std::path::PathBuf;
use subd_audio;
use subd_openai;

pub enum ImageRequestType {
    Dalle(subd_openai::dalle::DalleRequest),
    StableDiffusion(stable_diffusion::models::GenerateAndArchiveRequest),
}

pub async fn telephone(
    obs_client: &OBSClient,
    sink: &Sink,
    url: String,
    prompt: String,
    num_connections: u8,
    request_type: &ImageRequestType,
) -> Result<String, anyhow::Error> {
    let now: DateTime<Utc> = Utc::now();
    let folder = format!("telephone/{}", now.timestamp());
    let new_tele_folder = format!("./archive/{}", folder);
    let _ = create_dir(new_tele_folder.clone());

    // This shouldn't download an image always
    let og_file = format!("./archive/{}/original.png", folder);
    if let Err(e) = subd_image_utils::download_image_to_vec(
        url.clone(),
        Some(og_file.clone()),
    )
    .await
    {
        println!("Error Downloading Image: {} | {:?}", og_file.clone(), e);
    }

    let first_description = subd_openai::ask_gpt_vision2("", Some(&url))
        .await
        .map(|m| m)?;

    let description = format!("{} {}", first_description, prompt);
    println!("First GPT Vision Description: {}", description);

    let mut dalle_path = match &request_type {
        ImageRequestType::Dalle(ai_image_req) => {
            ai_image_req
                .generate_image(description, Some(folder.clone()), false)
                .await
        }
        ImageRequestType::StableDiffusion(ai_image_req) => {
            stable_diffusion::stable_diffusion_from_image(ai_image_req).await?
        }
    };
    if dalle_path == "".to_string() {
        return Err(anyhow!("Dalle Path is empty"));
    }

    let mut dalle_path_bufs = vec![];
    for _ in 1..num_connections {
        println!("\n\tAsking GPT VISION: {}", dalle_path.clone());
        let description =
            match subd_openai::ask_gpt_vision2(&dalle_path, None).await {
                Ok(desc) => desc,
                Err(e) => {
                    eprintln!("Error asking GPT Vision: {}", e);
                    continue;
                }
            };

        let prompt = format!("{} {}", description, prompt);
        println!("\n\tSaving Image to: {}", folder.clone());

        match &request_type {
            ImageRequestType::Dalle(ai_image_req) => {
                dalle_path = ai_image_req
                    .generate_image(prompt, Some(folder.clone()), false)
                    .await;
                if dalle_path != "".to_string() {
                    let dp = dalle_path.clone();
                    dalle_path_bufs.push(PathBuf::from(dp))
                }
            }
            ImageRequestType::StableDiffusion(og_req) => {
                let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
                let unique_identifier = format!("{}_screenshot.png", timestamp);
                let req = stable_diffusion::models::GenerateAndArchiveRequest {
                    prompt: og_req.prompt.clone(),
                    unique_identifier,
                    request_type:
                        stable_diffusion::models::RequestType::Img2ImgFile(
                            dalle_path,
                        ),
                    set_as_obs_bg: false,
                    additional_archive_dir: Some(folder.clone()),
                    strength: None,
                };
                let new_path =
                    stable_diffusion::stable_diffusion_from_image(&req).await?;
                dalle_path = new_path.clone();
            }
        }
    }

    // We take in an ID
    let _ =
        update_obs_telephone_scene(obs_client, og_file, dalle_path_bufs).await;
    let _ = subd_audio::play_sound(&sink, "8bitmackintro".to_string()).await;

    Ok(dalle_path)
}

// TODO: I don't like the name
pub async fn create_screenshot_img2img(
    obs_client: &OBSClient,
    filename: String,
    ai_image_req: &impl GenerateImage,
    prompt: String,
    source: String,
    archive_dir: Option<String>,
) -> Result<String> {
    let _ =
        obs_source::save_screenshot(&obs_client, &source, &filename).await?;

    // Do we want to add more???
    let new_description = format!("{}", prompt);

    let image_path = ai_image_req
        .generate_image(new_description, archive_dir, false)
        .await;

    if image_path == "".to_string() {
        return Err(anyhow!("Image Path is empty"));
    }

    // It's nice to print it off, to debug
    println!("Image Path: {}", image_path);
    Ok(image_path)
}

// TODO: I don't like the name
pub async fn create_screenshot_variation(
    _sink: &Sink,
    obs_client: &OBSClient,
    filename: String,
    request_type: ImageRequestType,
    prompt: String,
    source: String,
    archive_dir: Option<String>,
) -> Result<String> {
    // let _ = subd_audio::play_sound(&sink).await;

    let _ = obs_source::save_screenshot(&obs_client, &source, &filename).await;

    let description = subd_openai::ask_gpt_vision2(&filename, None).await?;

    let new_description = format!(
        "{} {} . The most important thing to focus on is: {}",
        prompt, description, prompt
    );

    let image_path = match request_type {
        ImageRequestType::Dalle(req) => {
            req.generate_image(new_description, archive_dir, false)
                .await
        }
        ImageRequestType::StableDiffusion(req) => {
            stable_diffusion::run_from_prompt(&req).await?
        }
    };

    if image_path == "".to_string() {
        return Err(anyhow!("Image Path is empty"));
    }

    // It's nice to print it off, to debug
    println!("Image Path: {}", image_path);
    Ok(image_path)
}

// Read in all files from DIR
// Exclude the
pub async fn update_obs_telephone_scene(
    obs_client: &OBSClient,
    og_image: String,
    image_variations: Vec<PathBuf>,
) -> Result<()> {
    let mut files: Vec<SlideshowFile> = vec![];
    let mut slideshow_files: Vec<SlideshowFile> = image_variations
        .iter()
        .map(|path_string| SlideshowFile {
            value: &path_string,
            hidden: false,
            selected: false,
        })
        .collect();
    files.append(&mut slideshow_files);

    let scene = "AIAssets".to_string();
    let source = "TelephoneSlideshow".to_string();
    let _ =
        obs_source::update_slideshow_source(obs_client, source.clone(), files)
            .await;
    let _ = obs_source::set_enabled(&scene, &source, true, &obs_client).await;

    let source = "OGTelephoneImage".to_string();
    let _ =
        obs_source::update_image_source(obs_client, source.clone(), og_image)
            .await;
    let _ = obs_source::set_enabled(&scene, &source, true, &obs_client).await;

    Ok(())
}

pub async fn old_obs_telephone_scene(
    obs_client: &OBSClient,
    id: String,
) -> Result<()> {
    let image_path = format!("/home/begin/code/subd/archive/telephone/{}/", id);
    let og_image = format!(
        "/home/begin/code/subd/archive/telephone/{}/original.png",
        id
    );

    let mut files: Vec<PathBuf> = vec![];
    for entry in fs::read_dir(image_path)? {
        let entry = entry?;
        let path = entry.path();
        files.push(path);
    }

    let slideshow_files: Vec<SlideshowFile> = files
        .iter()
        .map(|path_string| SlideshowFile {
            value: &path_string.as_path(),
            hidden: false,
            selected: false,
        })
        .collect();

    let source = "Telephone-Slideshow".to_string();
    let _ = obs_source::update_slideshow_source(
        obs_client,
        source,
        slideshow_files,
    )
    .await;

    let source = "OG-Telephone-Image".to_string();
    let _ = obs_source::update_image_source(obs_client, source, og_image).await;

    // TODO: Show the Telephoe-Image!
    // let scene = "TelephoneScene";
    // let _ = obs_scenes::change_scene(&obs_client, scene).await;
    return Ok(());
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_past_telephone() {}
}
