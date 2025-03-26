use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use subd_db::get_db_pool;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Subscription {
    id: String,
    status: String,
    #[serde(rename = "type")]
    type_field: String,
    version: String,
    condition: HashMap<String, String>,
    transport: Transport,
    created_at: String,
    cost: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventSub {
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    tier: Option<String>,
    is_gift: Option<bool>,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether to Enable All Features
    #[arg(long)]
    enable_all: bool,

    /// Whether to Enable Specific Features
    #[arg(long, value_delimiter = ' ', num_args = 1..)]
    enable: Vec<String>,
}

fn get_files_by_ext(directory: &str, extensions: &[&str]) -> Vec<String> {
    use std::fs;
    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if extensions.contains(&ext.as_str()) {
                            return path.file_name().map(|name| {
                                name.to_string_lossy().into_owned()
                            });
                        }
                    }
                }
                None
            })
            .collect(),
        Err(_) => vec![],
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "\n=== Starting AI Songs updater ===\n".cyan());

    env_logger::init();

    let args = Args::parse();
    // Create the event loop
    // Determine which features to enable
    let _features = if args.enable_all {
        vec!["fal".to_string()]
    } else {
        args.enable
    };

    let pool = get_db_pool().await;

    let _untagged_songs = ai_playlist::find_songs_without_tags(&pool).await?;
    // We need a list of all files in the folder
    let mp3s = get_files_by_ext("./ai_songs", &["mp3"]);

    for song in mp3s {
        let id = match Path::new(&song).file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem,
            None => {
                println!(
                    "Error: Unable to extract file stem for song: {}",
                    song
                );
                continue;
            }
        };

        // So we have a song ID
        println!("Song ID: {:?}", id);

        // So we don't hit the suno API to hard
        use std::time::Duration;

        let delay = Duration::from_millis(3000);
        std::thread::sleep(delay);

        let song_id = match Uuid::parse_str(id) {
            Ok(uuid) => uuid,
            Err(e) => {
                println!("Error parsing UUID for song {}: {}", id, e);
                continue;
            }
        };

        let db_record = ai_playlist::models::find_by_id(&pool, song_id).await;
        // If we have a DB record move on
        if db_record.is_ok() {
            log::info!("Song {} already in DB", id);
            continue;
        };

        log::info!("Creating new DB Entry for Song: {}", song_id);

        // We need to check if it's in the DB
        let created_at = sqlx::types::time::OffsetDateTime::now_utc();
        let res = match subd_suno::get_audio_information(id).await {
            Ok(r) => r,
            Err(_) => {
                println!("Error Getting Audio Information");
                continue;
            }
        };

        let new_song = ai_playlist::models::ai_songs::Model {
            song_id,
            title: res.title,
            prompt: res.metadata.prompt,
            username: "beginbot".to_string(),
            audio_url: res.audio_url,
            lyric: res.lyric,
            gpt_description_prompt: res.gpt_description_prompt,
            tags: res.tags,
            last_updated: Some(created_at),
            created_at: Some(created_at),
            downloaded: true,
        };

        // We aren't getting back the prompt info how we want
        log::info!("New Song: {:?}", new_song);
        log::info!("GPT Description: {:?}", new_song.gpt_description_prompt);

        match new_song.save(&pool).await {
            Ok(_) => {
                println!("Successfully saved song {} to database", song_id)
            }
            Err(e) => {
                println!("Error saving song {} to database: {}", song_id, e)
            }
        };

        //match audio_info {
        //    Ok(i) => {
        //        println!("\nAudio Info: {:?}\n", i);
        //        let tags = i.metadata.tags;
        //        let _ =
        //            ai_playlist::update_song_tags(&pool, song.song_id, tags)
        //                .await;
        //    }
        //    Err(e) => {
        //        println!("Error getting audio info for: {} {}", e, id)
        //    }
        // }
    }
    // Shared resource

    Ok(())
}
