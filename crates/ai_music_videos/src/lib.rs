use anyhow::{anyhow, Result};
use chrono::Utc;
use colored::Colorize;
use futures::future::join_all;
use sqlx::PgPool;
use std::sync::Arc;

pub mod scenes_builder;
pub mod utils;

pub async fn create_music_video_images_and_video(
    pool: &PgPool,
    id: String,
) -> Result<String> {
    println!("\tStarting to create NEW Music Video!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let ai_song = Arc::new(ai_song);

    let default_lyric = String::new();
    let lyrics = match ai_song.lyric.as_ref() {
        Some(l) => l,
        None => {
            println!("Error: Song lyrics are missing");
            &default_lyric
        }
    };
    let title = &ai_song.title;
    let scenes_prompts = scenes_builder::generate_scene_prompts(
        lyrics.to_string(),
        title.to_string(),
    )
    .await?;

    let music_video_folder = format!("./tmp/music_videos/{}", id);

    std::fs::create_dir_all(&music_video_folder)?;

    let image_files = match std::fs::read_dir(&music_video_folder) {
        Ok(files) => files,
        Err(_) => return Ok("".to_string()),
    };

    let highest_number = image_files
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .filter_map(|name| name.parse::<usize>().ok())
        .max()
        .unwrap_or(0);

    println!("Highest Number: {}", highest_number);

    // Create a vector of futures for concurrent execution
    let futures =
        scenes_prompts
            .scenes
            .iter()
            .enumerate()
            .map(|(index, scene)| {
                let ai_song = Arc::clone(&ai_song);
                let id = id.clone();

                let file_index = highest_number + (index + 1);
                async move {
                    create_image_from_prompt(
                        ai_song,
                        scene.image_prompt.clone(),
                        id,
                        file_index,
                    )
                    .await
                }
            });

    // Run all futures concurrently and collect the results
    let results: Vec<Result<String>> = join_all(futures).await;

    let mut video_results = Vec::new();
    for (index, result) in results.iter().enumerate() {
        match result {
            Ok(filename) => {
                println!("result: {}", filename);
                let scene = &scenes_prompts.scenes[index];

                let video_result = generate_runway_video_and_image(
                    &scene.image_prompt,
                    &scene.camera_move,
                    ai_song.song_id.to_string(),
                    index,
                );
                video_results.push(video_result);
            }
            Err(e) => eprintln!("Error processing image: {:?}", e),
        }
    }

    let video_filenames: Vec<String> = join_all(video_results)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let music_video_folder = format!("./tmp/music_videos/{}", id);
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let output_file =
        format!("{}/{}_{}", music_video_folder, timestamp, "final_video.mp4");
    combine_videos(video_filenames, &output_file)?;

    Ok(output_file)
}

pub async fn create_video_from_image(
    song_id: &str,
    image_name: &str,
    path_string: &str,
) -> Result<()> {
    println!("Describing the Image_name: {}", path_string);
    let description = subd_openai::ask_gpt_vision2(path_string, None).await?;
    println!("Description: {}", description);

    let prompt = format!("Describe how this scene could transform based on the following description, add random fun transitions, make the description concise. Description: {}", description);
    let scene = scenes_builder::generate_scene_from_prompt(prompt).await?;
    let video_prompt =
        format!("{} {}", scene.scene_description, scene.camera_move);

    println!("Video Prompt: {}", video_prompt);

    let _ =
        generate_runway_video_from_image(song_id, image_name, &video_prompt)
            .await;
    Ok(())
}

pub async fn create_music_video_image(
    pool: &PgPool,
    id: String,
    prompt: Option<String>,
    index_offset: Option<i64>,
) -> Result<String> {
    println!("\tAttempting to create Music Video Image!");

    let ai_song = match ai_playlist::find_song_by_id(pool, &id).await {
        Ok(song) => song,
        Err(e) => {
            println!("Error finding song by id: {} | {:?}", id, e);
            return Err(e);
        }
    };
    let ai_song = Arc::new(ai_song);

    let image_prompt = match prompt {
        Some(p) => {
            format!(
                "{} in the context of a music video titled: {}",
                p, ai_song.title
            )
        }
        None => {
            let lyrics = match ai_song.lyric.as_ref() {
                Some(l) => l,
                None => {
                    println!("Error: Song lyrics are missing");
                    return Err(anyhow::anyhow!("Song lyrics are missing"));
                }
            };
            let title = &ai_song.title;
            let scene = match scenes_builder::generate_scene_prompt(
                lyrics.to_string(),
                title.to_string(),
            )
            .await
            {
                Ok(s) => s,
                Err(e) => {
                    println!("Error generating scene prompt: {:?}", e);
                    return Err(e);
                }
            };
            scene.image_prompt.clone()
        }
    };

    let music_video_folder = format!("./tmp/music_videos/{}", id);

    if let Err(e) = std::fs::create_dir_all(&music_video_folder) {
        println!("Error creating directory: {:?}", e);
        return Err(e.into());
    }

    let image_files = match std::fs::read_dir(&music_video_folder) {
        Ok(files) => files,
        Err(e) => {
            println!("Error reading directory: {:?}", e);
            return Err(e.into());
        }
    };

    let highest_number = image_files
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .filter_map(|name| name.parse::<usize>().ok())
        .max()
        .unwrap_or(0);

    let file_index = match index_offset {
        Some(offset) => highest_number + offset as usize,
        None => highest_number + 1,
    };
    create_image_from_prompt(ai_song, image_prompt, id, file_index).await
}

async fn create_image_from_prompt(
    ai_song: Arc<ai_playlist::models::ai_songs::Model>,
    prompt: String,
    id: String,
    index: usize,
) -> Result<String> {
    println!(
        "{} - {}",
        "Creating Image for Prompt: {}".cyan(),
        prompt.green()
    );

    let folder = format!("./tmp/music_videos/{}", id);
    let prompt = format!("{} {}", ai_song.title, prompt);

    println!("Calling create_and_save_image w/ {}", index);
    let images = fal_ai::create_and_save_image(
        &prompt,
        Some(&index.to_string()),
        Some(&folder.clone()),
    )
    .await?;
    let first_image = images.first().ok_or_else(|| anyhow!("No Image"))?;
    Ok(first_image.to_string())
}

async fn generate_runway_video_from_image(
    song_id: &str,
    image_name: &str,
    video_prompt: &str,
) -> Result<String> {
    let folder = format!("./tmp/music_videos/{}", song_id);
    let filename = fal_ai::create_runway_video_from_image(
        video_prompt,
        image_name,
        Some(&folder),
    )
    .await?;
    Ok(filename)
}

async fn generate_runway_video_and_image(
    image_prompt: &str,
    video_prompt: &str,
    id: String,
    index: usize,
) -> Result<String> {
    let folder = format!("./tmp/music_videos/{}", id);
    let images = fal_ai::create_and_save_image(
        image_prompt,
        Some(&index.to_string()),
        Some(&folder.clone()),
    )
    .await?;
    let first_image = images.first().ok_or_else(|| anyhow!("No Image"))?;
    println!("Image: {}", first_image);
    let filename = fal_ai::create_runway_video_from_image(
        video_prompt,
        first_image,
        Some(&folder),
    )
    .await?;
    Ok(filename)
}

// ================================

fn combine_videos(video_chunks: Vec<String>, output_file: &str) -> Result<()> {
    let mut input_files = String::new();
    for chunk in video_chunks {
        input_files.push_str(&format!("file '{}'\n", chunk));
    }

    let temp_file = "temp_file_list.txt";
    std::fs::write(temp_file, input_files)?;

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            temp_file,
            "-c",
            "copy",
            output_file,
        ])
        .status()?;

    std::fs::remove_file(temp_file)?;

    if status.success() {
        println!("Combined video created successfully: {}", output_file);
        Ok(())
    } else {
        Err(anyhow!("Failed to combine videos"))
    }
}

// This belongs somewhere else
fn _create_slideshow_from_images(song_id: &str) -> Result<String> {
    let output_file = format!("./tmp/music_videos/{}/video.mp4", song_id);
    let input_pattern = format!("./tmp/music_videos/{}/*.jpg", song_id);

    utils::remove_small_images(song_id, 10_000)?;

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate",
            "1/2",
            "-pattern_type",
            "glob",
            "-i",
            &input_pattern,
            "-c:v",
            "libx264",
            "-r",
            "30",
            "-pix_fmt",
            "yuv420p",
            &output_file,
        ])
        .status()?;

    if status.success() {
        println!("Video created successfully: {}", output_file);
        Ok(output_file)
    } else {
        Err(anyhow!("Failed to create video"))
    }
}

#[cfg(test)]
mod tests;
