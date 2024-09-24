use anyhow::{anyhow, Result};
use colored::Colorize;
use sqlx::PgPool;
use std::path::Path;
use subd_db;
use uuid::Uuid;

pub async fn create_music_video(pool: &PgPool, id: String) -> Result<String> {
    println!("\tIt's Music Video time!");

    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;
    let lyric_chunks = get_lyric_chunks(&ai_song.lyric)?;

    create_images_for_lyrics(&ai_song, &lyric_chunks).await?;
    let output_file = create_video(&id)?;

    Ok(output_file)
}

// this can fail
fn get_lyric_chunks(lyric: &Option<String>) -> Result<Vec<String>> {
    let lyric = lyric
        .as_ref()
        .ok_or_else(|| anyhow!("No Lyrics to parse"))?;
    let chunks = lyric
        .split_whitespace()
        .collect::<Vec<_>>()
        .chunks(20)
        .map(|chunk| chunk.join(" "))
        .collect();
    Ok(chunks)
}

async fn create_images_for_lyrics(
    ai_song: &ai_playlist::models::ai_songs::Model,
    lyric_chunks: &[String],
) -> Result<()> {
    for (index, lyric) in lyric_chunks.iter().enumerate() {
        println!(
            "{} - {}",
            "Creating Image for Lyric Chunk: {}".cyan(),
            lyric.green()
        );

        fal_ai::create_image_for_music_video(
            &ai_song.song_id.to_string(),
            &format!("{} {}", ai_song.title, lyric),
            index + 1,
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

mod tests {
    use super::*;
    use ai_playlist::models::ai_songs;

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
