use anyhow::anyhow;
use anyhow::Result;
use rodio::Decoder;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SunoResponse {
    pub id: String,
    pub video_url: String,
    pub audio_url: String,
    pub image_url: String,
    pub lyric: Option<String>,
    pub image_large_url: Option<String>,
    pub is_video_pending: Option<bool>,

    #[serde(default)]
    pub major_model_version: String,
    pub model_name: String,

    #[serde(default)]
    pub metadata: Metadata,

    #[serde(default)]
    pub display_name: String,

    #[serde(default)]
    pub handle: String,
    #[serde(default)]
    pub is_handle_updated: bool,
    #[serde(default)]
    pub avatar_image_url: String,
    #[serde(default)]
    pub is_following_creator: bool,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub play_count: i32,
    #[serde(default)]
    pub upvote_count: i32,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub tags: String,
    pub prompt: String,
    pub gpt_description_prompt: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub duration: f64,
    pub refund_credits: bool,
    pub stream: bool,
}

pub async fn play_audio(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    sink: &Sink,
    id: &str,
    user_name: &str,
) -> Result<()> {
    println!("\tQueuing {}", id);
    let info = format!("@{} added {} to Queue", user_name, id);
    let _ = send_message(&twitch_client, info).await;

    let file_name = format!("ai_songs/{}.mp3", id);
    let mp3 = match File::open(&file_name) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening sound file: {}", e);
            return Ok(());
        }
    };
    let file = BufReader::new(mp3);
    println!("\tPlaying Audio {}", id);

    let uuid_id = uuid::Uuid::parse_str(id)?;

    println!("Adding to Playlist");
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
    ai_playlist::mark_song_as_played(pool, uuid_id).await?;

    let _ = play_sound_instantly(sink, file).await;

    Ok(())
}

pub async fn get_audio_information(id: &str) -> Result<SunoResponse> {
    let base_url = "http://localhost:3000";
    // This actually works
    // let base_url = "https://api.suno.ai";
    let url = format!("{}/api/get?ids={}", base_url, id);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let suno_response: Vec<SunoResponse> = response.json().await?;

    suno_response
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No audio information found"))
}

async fn play_sound_instantly(
    sink: &Sink,
    file: BufReader<File>,
) -> Result<()> {
    match Decoder::new(BufReader::new(file)) {
        Ok(v) => {
            // This clear() seems to cause problems
            // but it might be because we didn't pause enought before the append
            // but that also would suck
            // sink.clear();

            println!("\tAppending Sound");
            sink.append(v);

            // If we sleep_until_end here,
            // it blocks other commands in this ai_handler
            // we might want to consider careful how to divide up these functions
            // and share the proper handlers
            // sink.sleep_until_end()
        }
        Err(e) => {
            eprintln!("Error decoding sound file: {}", e);
            return Err(anyhow!("Error decoding sound file: {}", e));
        }
    };

    Ok(())
}
