use anyhow::Result;
use std::path::PathBuf;
use std::path::Path;
use crate::images;
use crate::openai;
use chrono::{DateTime, Utc};
use crate::audio;
use crate::dalle::GenerateImage;
use crate::dalle;
use crate::obs_scenes;
use crate::obs_source;
use obws::Client as OBSClient;
use obws::requests::custom::source_settings::SlideshowFile;
use obws::requests::custom::source_settings::ImageSource;
use std::fs::create_dir;
use rodio::*;

pub async fn telephone(
    obs_client: &OBSClient,
    sink: &Sink,
    url: String,
    prompt: String,
    num_connections: u8,
    ai_image_req: &impl GenerateImage,
) -> Result<String, anyhow::Error> {
    let now: DateTime<Utc> = Utc::now();
    let folder = format!("telephone/{}", now.timestamp());
    let new_tele_folder = format!("./archive/{}", folder);
    let _ = create_dir(new_tele_folder.clone());

    println!("new_dir: {:?}", new_tele_folder.clone());

    let og_file =
        format!("/home/begin/code/subd/archive/{}/original.png", folder);
    if let Err(e) =
        images::download_image(url.clone(), og_file.clone()).await
    {
        println!(
            "Error Downloading Image: {} | {:?}",
            og_file.clone(),
            e
        );
    }
    let image_settings = ImageSource {
        file: &Path::new(&og_file),
        unload: true,
    };
    let set_settings = obws::requests::inputs::SetSettings {
        settings: &image_settings,
        input: "OG-Telephone-Image",
        overlay: Some(true),
    };
    let _ = obs_client.inputs().set_settings(set_settings).await;

    // let first_description = match ask_gpt_vision2(&archive_file, None).await {
    let first_description = match openai::ask_gpt_vision2("", Some(&url)).await {
        Ok(description) => description,
        Err(e) => {
            eprintln!("Error asking GPT Vision for description: {}", e);
            return Err(e.into());
        }
    };

    let description = format!("{} {}", first_description, prompt);
    println!("First Telescope: {}", description);
    let mut dalle_path = ai_image_req
        .generate_image(description, Some(folder.clone()), false)
        .await;
    if dalle_path == "".to_string() {
        return Err(anyhow::anyhow!("Dalle Path is empty"));
    }

    // Create our list of files for the slideshow
    let dp = dalle_path.clone();
    let fp = Path::new(&dp);
    let slideshow_file = SlideshowFile {
        value: fp,
        hidden: false,
        selected: true,
    };
    let mut files: Vec<SlideshowFile> = vec![slideshow_file];

    let mut dalle_path_bufs = vec![];

    for _ in 1..num_connections {
        println!("\n\tAsking GPT VISION: {}", dalle_path.clone());
        let description = match openai::ask_gpt_vision2(&dalle_path, None).await {
            Ok(description) => description,
            Err(e) => {
                eprintln!("Error asking GPT Vision: {}", e);
                continue;
            }
        };

        let prompt = format!("{} {}", description, prompt);
        let req = dalle::DalleRequest {
            prompt: prompt.clone(),
            username: "beginbot".to_string(),
            amount: 1,
        };
        println!("\n\tSaving Image to: {}", folder.clone());

        dalle_path = req
            .generate_image(prompt, Some(folder.clone()), false)
            .await;
        // Do we need to add information here?
        if dalle_path != "".to_string() {
            let dp = dalle_path.clone();
            dalle_path_bufs.push(PathBuf::from(dp))
        }
    }

    let mut slideshow_files: Vec<SlideshowFile> = dalle_path_bufs
        .iter()
        .map(|path_string| SlideshowFile {
            value: &path_string,
            hidden: false,
            selected: false,
        })
        .collect();

    files.append(&mut slideshow_files);

    let slideshow_settings =
        obws::requests::custom::source_settings::Slideshow {
            files: &files,
            ..Default::default()
        };
    let set_settings = obws::requests::inputs::SetSettings {
        settings: &slideshow_settings,
        input: "Telephone-Slideshow",
        overlay: Some(true),
    };
    let _ = obs_client.inputs().set_settings(set_settings).await;


    let _ = obs_scenes::change_scene(&obs_client, "TelephoneScene").await;
    let _ = audio::play_sound(&sink, "8bitmackintro".to_string()).await;
    Ok(dalle_path)
}


// TODO: I don't like the name
pub async fn create_screenshot_variation(
    sink: &Sink,
    obs_client: &OBSClient,
    filename: String,
    ai_image_req: &impl GenerateImage,
    prompt: String,
    source: String,
) -> Result<String, String> {
    // let _ = audio::play_sound(&sink).await;

    let _ = obs_source::save_screenshot(&obs_client, &source, &filename).await;

    let description = openai::ask_gpt_vision2(&filename, None)
        .await
        .map_err(|e| e.to_string())?;

    let new_description = format!(
        "{} {} . The most important thing to focus on is: {}",
        prompt, description, prompt
    );

    let dalle_path = ai_image_req
        .generate_image(new_description, Some("timelapse".to_string()), false)
        .await;

    if dalle_path == "".to_string() {
        return Err("Dalle Path is empty".to_string());
    }

    println!("Dalle Path: {}", dalle_path);
    Ok(dalle_path)
}
