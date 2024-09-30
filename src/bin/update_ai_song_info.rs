use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use subd_db::get_db_pool;

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
    let untagged_songs = ai_playlist::find_songs_without_tags(&pool).await?;

    for song in untagged_songs {
        println!("Updating song: {}", song.song_id);
        let id = song.song_id.to_string();
        let audio_info = subd_suno::get_audio_information(&id).await;

        match audio_info {
            Ok(i) => {
                println!("\nAudio Info: {:?}\n", i);
                let tags = i.metadata.tags;
                let _ =
                    ai_playlist::update_song_tags(&pool, song.song_id, tags)
                        .await;
            }
            Err(e) => {
                println!("Error getting audio info for: {} {}", e, id)
            }
        }
    }
    // Shared resource

    Ok(())
}
