use anyhow::{anyhow, Result};
use chrono::Utc;
use colored::Colorize;
use futures::future::join_all;
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;

// ==============================================================
use instruct_macros::InstructMacro;
use instruct_macros_types::Parameter;
use instruct_macros_types::{ParameterInfo, StructInfo};
use instructor_ai::from_openai;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{self, ChatCompletionRequest},
    common::GPT3_5_TURBO,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct MusicVideoScenes {
    scenes: Vec<MusicVideoScene>,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct MusicVideoScene {
    image_prompt: String,
    camera_move: String,
    image_name: Option<String>,
}
// ==============================================================

async fn generate_scene_prompts(
    lyrics: String,
    title: String,
) -> Result<MusicVideoScenes> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let prompt = format!(
        "Describe 5 scenes for a Music Video: image_prompt and camera_move based on the following Lyrics: {} and Title: {}. They should be fun scenes that stick to an overall theme based on the title.",
        lyrics, title);

    println!("\tUsing the Prompt: {}", prompt);

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(prompt)),
            name: None,
        }],
    );

    let result =
        match instructor_client.chat_completion::<MusicVideoScenes>(req, 3) {
            Ok(scenes) => scenes,
            Err(e) => {
                eprintln!("Error generating scene prompts: {:?}", e);
                return Err(anyhow!("Failed to generate scene prompts"));
            }
        };

    println!("{:?}", result);
    Ok(result)
}
pub async fn create_music_video_images(
    pool: &PgPool,
    id: String,
) -> Result<String> {
    println!("\tStarting to create NEW Music Video!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let ai_song = Arc::new(ai_song);

    let lyrics = ai_song.lyric.as_ref().unwrap();
    let title = &ai_song.title;
    let scenes_prompts =
        generate_scene_prompts(lyrics.to_string(), title.to_string()).await?;

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
                    create_image_from_lyric(
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

                let video_result = generate_runway_video_for_image(
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

pub async fn create_music_video_2(pool: &PgPool, id: String) -> Result<String> {
    println!("\tStarting to create NEW Music Video!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let ai_song = Arc::new(ai_song);

    let filtered_lyric = ai_song.lyric.as_ref().map(|lyric| {
        lyric
            .lines()
            .filter(|line| !line.trim().starts_with('['))
            .collect::<Vec<_>>()
            .join("\n")
    });
    let lyric_chunks = get_lyric_chunks(&filtered_lyric, 30)?;

    let music_video_folder = format!("./tmp/music_videos/{}", id);

    std::fs::create_dir_all(&music_video_folder)?;

    let image_files = match std::fs::read_dir(&music_video_folder) {
        Ok(files) => files,
        Err(_) => return Ok("0".to_string()),
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
    let futures = lyric_chunks.into_iter().enumerate().map(|(index, lyric)| {
        let ai_song = Arc::clone(&ai_song);
        let id = id.clone();

        let file_index = highest_number + (index + 1);
        async move { process_lyric_chunk(ai_song, lyric, id, file_index).await }
    });

    // Run all futures concurrently and collect the results
    let results: Vec<Result<String>> = join_all(futures).await;

    // I don't always to do this

    // Collect filenames or handle errors
    let mut video_chunks: Vec<String> = Vec::new();
    for result in results {
        match result {
            Ok(filename) => video_chunks.push(filename),
            Err(e) => {
                eprintln!("Error processing lyric chunk: {:?}", e);
                return Err(e);
            }
        }
    }

    let music_video_folder = format!("./tmp/music_videos/{}", id);
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let output_file =
        format!("{}/{}_{}", music_video_folder, timestamp, "final_video.mp4");
    combine_videos(video_chunks, &output_file)?;

    Ok(output_file)
}

async fn create_image_from_lyric(
    ai_song: Arc<ai_playlist::models::ai_songs::Model>,
    lyric: String,
    id: String,
    index: usize,
) -> Result<String> {
    println!(
        "{} - {}",
        "Creating Image for Lyric Chunk: {}".cyan(),
        lyric.green()
    );

    //
    let folder = format!("./tmp/music_videos/{}", id);
    let prompt = format!("{} {}", ai_song.title, lyric);
    let images = fal_ai::create_from_fal_api_return_filename(
        &prompt,
        Some(folder.clone()),
        index.to_string(),
    )
    .await?;
    let first_image = images.get(0).ok_or_else(|| anyhow!("No Image"))?;
    Ok(first_image.to_string())
}

async fn generate_runway_video_for_image(
    image_prompt: &str,
    video_prompt: &str,
    id: String,
    index: usize,
) -> Result<String> {
    let folder = format!("./tmp/music_videos/{}", id);
    let images = fal_ai::create_from_fal_api_return_filename(
        &image_prompt,
        Some(folder.clone()),
        index.to_string(),
    )
    .await?;
    let first_image = images.get(0).ok_or_else(|| anyhow!("No Image"))?;
    println!("Image: {}", first_image);
    let filename = fal_ai::create_runway_video_from_image(
        &video_prompt,
        first_image,
        Some(folder.clone()),
    )
    .await?;
    Ok(filename)
}

async fn process_lyric_chunk(
    ai_song: Arc<ai_playlist::models::ai_songs::Model>,
    lyric: String,
    id: String,
    index: usize,
) -> Result<String> {
    println!(
        "{} - {}",
        "Creating Image for Lyric Chunk: {}".cyan(),
        lyric.green()
    );

    //
    let folder = format!("./tmp/music_videos/{}", id);
    let prompt = format!("{} {}", ai_song.title, lyric);
    // Pass the index into here
    let images = fal_ai::create_from_fal_api_return_filename(
        &prompt,
        Some(folder.clone()),
        index.to_string(),
    )
    .await?;
    let first_image = images.get(0).ok_or_else(|| anyhow!("No Image"))?;
    println!("Image: {}", first_image);
    let filename =
        fal_ai::create_video_from_image(first_image, Some(folder.clone()))
            .await?;
    Ok(filename)
}

fn combine_videos(video_chunks: Vec<String>, output_file: &str) -> Result<()> {
    let mut input_files = String::new();
    for chunk in video_chunks {
        input_files.push_str(&format!("file '{}'\n", chunk));
    }

    let temp_file = "temp_file_list.txt";
    std::fs::write(temp_file, input_files)?;

    let status = std::process::Command::new("ffmpeg")
        .args(&[
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

// this can fail
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

fn create_video(song_id: &str) -> Result<String> {
    let output_file = format!("./tmp/music_videos/{}/video.mp4", song_id);
    let input_pattern = format!("./tmp/music_videos/{}/*.jpg", song_id);

    remove_small_images(song_id, 10_000)?;

    let status = std::process::Command::new("ffmpeg")
        .args(&[
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

#[cfg(test)]
mod tests {
    use super::*;
    use ai_playlist::models::ai_songs;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_highest_file_number() {
        let uuid_str = "0833d255-607f-4b74-bea9-4818f032140a";
        let id = Uuid::parse_str(uuid_str).unwrap();
        let music_video_folder = format!("../../tmp/music_videos/{}", id);
        let highest_number = std::fs::read_dir(&music_video_folder)
            .unwrap()
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
        assert_eq!(1727998927, highest_number);
    }

    #[ignore]
    #[tokio::test]
    async fn test_create_music_video() {
        let pool = subd_db::get_test_db_pool().await;

        let fake_uuid = Uuid::new_v4();
        let ai_song = ai_songs::Model::new(
            fake_uuid,
            "title".into(),
            "tags".into(),
            "prompt".into(),
            "username".into(),
            "audio_url".into(),
            "gpt_description_prompt".into(),
            Some("Lyrics Hooray!".to_string()),
            None,
            None,
            false,
        );

        ai_song.save(&pool).await.unwrap();
        let id = format!("{}", fake_uuid);
        let res = create_music_video(&pool, id).await.unwrap();
        // OK
    }
}
