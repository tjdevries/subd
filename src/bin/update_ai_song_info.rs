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
use std::{thread, time};

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "\n=== Starting AI Songs updater ===\n".cyan());

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

    // TODO: remove unwraps
    for song in mp3s {
        let id = Path::new(&song).file_stem().unwrap().to_str().unwrap();
        println!("Song Info: {:?}", id);
        //let delay = time::Duration::from_millis(50);
        //thread::sleep(delay);

        let song_id = Uuid::parse_str(&id).unwrap();
        let db_record = ai_playlist::models::find_by_id(&pool, song_id).await;
        // If we have a DB record move on
        if let Ok(_) = db_record {
            continue;
        };

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
            tags: res.metadata.tags,
            prompt: res.metadata.prompt,
            username: "beginbot".to_string(),
            audio_url: res.audio_url,
            lyric: res.lyric,
            gpt_description_prompt: res.metadata.gpt_description_prompt,
            last_updated: Some(created_at),
            created_at: Some(created_at),
            downloaded: true,
        };
        println!("Result: {:?}", new_song);

        //
        //// Save the song if it doesn't already exist
        let _ = new_song.save(&pool).await;

        // We need to now call out and create a new ai_song

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
