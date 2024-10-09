use anyhow::{anyhow, Result};
use chrono::Utc;
use colored::Colorize;
use futures::future::join_all;
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;

pub mod scenes_builder;

#[cfg(test)]
mod tests;

pub async fn create_video_from_image(
    song_id: &str,
    image_name: &str,
    path_string: &str,
) -> Result<()> {
    // I can pass that in
    // Do I have the exact filepath here???
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

    let file_index = highest_number + 1;
    create_image_from_prompt(ai_song, image_prompt, id, file_index).await
}

pub async fn create_music_video_images_and_video(
    pool: &PgPool,
    id: String,
) -> Result<String> {
    println!("\tStarting to create NEW Music Video!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let ai_song = Arc::new(ai_song);

    let lyrics = ai_song.lyric.as_ref().unwrap();
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

async fn create_image_from_prompt(
    ai_song: Arc<ai_playlist::models::ai_songs::Model>,
    prompt: String,
    id: String,
    index: usize,
) -> Result<String> {
    println!(
        "{} - {}",
        "Creating Image for Lyric Chunk: {}".cyan(),
        prompt.green()
    );

    let folder = format!("./tmp/music_videos/{}", id);
    let prompt = format!("{} {}", ai_song.title, prompt);
    let images = fal_ai::create_from_fal_api_return_filename(
        &prompt,
        Some(folder.clone()),
        index.to_string(),
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
        Some(folder.clone()),
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
    let images = fal_ai::create_from_fal_api_return_filename(
        image_prompt,
        Some(folder.clone()),
        index.to_string(),
    )
    .await?;
    let first_image = images.first().ok_or_else(|| anyhow!("No Image"))?;
    println!("Image: {}", first_image);
    let filename = fal_ai::create_runway_video_from_image(
        video_prompt,
        first_image,
        Some(folder.clone()),
    )
    .await?;
    Ok(filename)
}

pub async fn create_music_video(pool: &PgPool, id: String) -> Result<String> {
    println!("\tIt's Music Video time!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let filtered_lyric = ai_song.lyric.as_ref().map(|lyric| {
        lyric
            .lines()
            .filter(|line| !line.trim().starts_with('['))
            .collect::<Vec<_>>()
            .join("\n")
    });
    let lyric_chunks = get_lyric_chunks(&filtered_lyric, 20)?;

    create_images_for_lyrics(&ai_song, &lyric_chunks).await?;
    let output_file = create_video(&id)?;

    Ok(output_file)
}

fn get_lyric_chunks(
    lyric: &Option<String>,
    chunksize: usize,
) -> Result<Vec<String>> {
    let lyric = lyric
        .as_ref()
        .ok_or_else(|| anyhow!("No Lyrics to parse"))?;
    let chunks = lyric
        .split_whitespace()
        .collect::<Vec<_>>()
        .chunks(chunksize)
        .map(|chunk| chunk.join(" "))
        .collect();
    Ok(chunks)
}

async fn create_images_for_lyrics(
    ai_song: &ai_playlist::models::ai_songs::Model,
    lyric_chunks: &[String],
) -> Result<()> {
    for lyric in lyric_chunks {
        println!(
            "{} - {}",
            "Creating Image for Lyric Chunk: {}".cyan(),
            lyric.green()
        );

        fal_ai::create_image_for_music_video(
            &ai_song.song_id.to_string(),
            &format!("{} {}", ai_song.title, lyric),
        )
        .await?;
    }
    Ok(())
}

fn remove_small_images(song_id: &str, min_size: u64) -> Result<()> {
    let dir_path = format!("./tmp/music_videos/{}", song_id);
    let dir = Path::new(&dir_path);

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("jpg")
        {
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() <= min_size {
                println!("Removing: {:?}", path);
                std::fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

// ================================

// This belongs somewhere else
fn create_video(song_id: &str) -> Result<String> {
    let output_file = format!("./tmp/music_videos/{}/video.mp4", song_id);
    let input_pattern = format!("./tmp/music_videos/{}/*.jpg", song_id);

    remove_small_images(song_id, 10_000)?;

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
