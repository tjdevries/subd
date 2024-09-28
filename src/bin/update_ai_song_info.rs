use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use elevenlabs_api::{Auth, Elevenlabs};
use obs_service::obs::create_obs_client;
use obws::Client;
use serde::{Deserialize, Serialize};
use server::handlers;
use std::collections::HashMap;
use subd_db::get_db_pool;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

// #[derive(Serialize, Deserialize, Debug)]
// struct EventSubRoot {
//     subscription: Subscription,
//     event: Option<EventSub>,
//     challenge: Option<String>,
// }

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

struct AppResources {
    obs_client: Client,
    sink: rodio::Sink,
    twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    elevenlabs: Elevenlabs,
}

impl AppResources {
    /// Creates a new instance of `AppResources` with fresh resources.
    async fn new(stream_handle: &rodio::OutputStreamHandle) -> Result<Self> {
        // Initialize OBS client
        let obs_client = create_obs_client().await?;

        // This is how we play audio
        let sink = rodio::Sink::try_new(stream_handle)?;

        // Initialize Twitch client
        let twitch_config = get_chat_config();
        let (_, twitch_client) = TwitchIRCClient::<
            SecureTCPTransport,
            StaticLoginCredentials,
        >::new(twitch_config);

        // Initialize ElevenLabs client
        let elevenlabs_auth = Auth::from_env().unwrap();
        let elevenlabs =
            Elevenlabs::new(elevenlabs_auth, "https://api.elevenlabs.io/v1/");

        Ok(Self {
            obs_client,
            sink,
            twitch_client,
            elevenlabs,
        })
    }
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
