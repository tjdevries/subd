use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use sqlx::PgPool;
use std::path::Path;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsDownloader {
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

enum Command {
    CreateMusicVideo { id: String },
    Download { id: String },
    CreateSong { prompt: String },
    Unknown,
}

#[async_trait]
impl EventHandler for AISongsDownloader {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        while let Ok(event) = rx.recv().await {
            if let Event::UserMessage(msg) = event {
                if let Err(err) = handle_requests(
                    &tx,
                    &self.obs_client,
                    &self.twitch_client,
                    &self.pool,
                    msg,
                )
                .await
                {
                    eprintln!("Error handling request: {}", err);
                }
            }
        }
        Ok(())
    }
}

/// Handles incoming requests based on the parsed command.
pub async fn handle_requests(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    msg: UserMessage,
) -> Result<()> {
    // Ignore messages from the bot itself
    if ["nightbot"].contains(&msg.user_name.as_str()) {
        return Ok(());
    }

    match parse_command(&msg) {
        Command::Download { id } => {
            handle_download_command(twitch_client, tx, msg.user_name, id).await
        }
        Command::CreateSong { prompt } => {
            handle_create_song_command(
                twitch_client,
                pool,
                tx,
                msg.user_name,
                prompt,
            )
            .await
        }
        Command::Unknown => Ok(()),
        Command::CreateMusicVideo { id } => {
            let filename = handle_create_music_video_command(
                twitch_client,
                pool,
                tx,
                msg.user_name,
                id,
            )
            .await?;
            let path = std::fs::canonicalize(&filename)?;
            let full_path = path
                .into_os_string()
                .into_string()
                .map_err(|_| anyhow!("Failed to convert path to string"))?;

            // path.file_name().and_then
            let source = "music-video".to_string();
            let _ = obs_service::obs_source::set_enabled(
                "AIFriends",
                &source.clone(),
                false,
                obs_client,
            )
            .await;
            let _ = obs_service::obs_source::update_video_source(
                obs_client,
                source.clone(),
                full_path,
            )
            .await;
            let _ = obs_service::obs_source::set_enabled(
                "AIFriends",
                &source,
                true,
                obs_client,
            )
            .await;
            Ok(())
            // obs_client

            // We have the id and the video is complete
            // I could use OBS client, to play the video
        }
    }
}

/// Parses a user's message into a `Command`.
fn parse_command(msg: &UserMessage) -> Command {
    let mut words = msg.contents.split_whitespace();
    match words.next() {
        Some("!create_music_video") => {
            if let Some(id) = words.next() {
                Command::CreateMusicVideo { id: id.to_string() }
            } else {
                Command::Unknown
            }
        }
        Some("!download") => {
            if let Some(id) = words.next() {
                Command::Download { id: id.to_string() }
            } else {
                Command::Unknown
            }
        }
        Some("!create_song") | Some("!song") => {
            let prompt = words.collect::<Vec<_>>().join(" ");
            Command::CreateSong { prompt }
        }
        _ => Command::Unknown,
    }
}
/// Handles the `!download` command.
async fn handle_download_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    tx: &broadcast::Sender<Event>,
    user_name: String,
    id: String,
) -> Result<()> {
    subd_suno::download_and_play(twitch_client, tx, user_name, &id).await
}

/// Handles the `!create_music_video` and `!music_video` commands.
async fn handle_create_music_video_command(
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &PgPool,
    _tx: &broadcast::Sender<Event>,
    _user_name: String,
    id: String,
) -> Result<String> {
    println!("\tIt's Music Video time!");
    let ai_song = ai_playlist::find_song_by_id(pool, &id).await?;

    // let chunk_count = ai_song.lyric.split_whitespace().count() / 10;
    let lyric_chunks = ai_song
        .lyric
        .ok_or(anyhow!("No Lyrics to parse"))?
        .split_whitespace()
        .collect::<Vec<_>>()
        .chunks(20)
        .map(|chunk| chunk.join(" "))
        .collect::<Vec<String>>();

    for (index, lyric) in lyric_chunks.into_iter().enumerate() {
        println!(
            "{} - {}",
            "Creating Image for Lyric Chunk: {}".cyan(),
            lyric.green()
        );
        // I should return and do something
        let _ = fal_ai::create_image_for_music_video(
            format!("{}", ai_song.song_id),
            format!("{} {}", ai_song.title, lyric),
            index + 1,
        )
        .await;
    }

    let output_file =
        format!("./tmp/music_videos/{}/video.mp4", ai_song.song_id);
    let input_pattern = format!("./tmp/music_videos/{}/*.jpg", ai_song.song_id);
    let _ = std::fs::read_dir(Path::new(&input_pattern).parent().unwrap())
        .unwrap()
        .for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "jpg" {
                let metadata = std::fs::metadata(&path).unwrap();
                if metadata.len() <= 10_000 {
                    println!("Removing: {:?}", path);
                    std::fs::remove_file(&path).unwrap();
                }
            }
        });

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
    } else {
        return Err(anyhow!("Failed to create video"));
    }

    return Ok(output_file);
}

async fn handle_create_song_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    tx: &broadcast::Sender<Event>,
    user_name: String,
    prompt: String,
) -> Result<()> {
    println!("\tIt's Song time!");
    let data = subd_suno::AudioGenerationData {
        prompt,
        make_instrumental: false,
        wait_audio: true,
    };
    match subd_suno::generate_audio_by_prompt(data).await {
        Ok(json_response) => {
            println!("JSON Response: {:#?}", json_response);
            subd_suno::parse_suno_response_download_and_play(
                twitch_client,
                pool,
                tx,
                json_response.clone(),
                0,
                user_name.clone(),
            )
            .await?;
            subd_suno::parse_suno_response_download_and_play(
                twitch_client,
                pool,
                tx,
                json_response,
                1,
                user_name,
            )
            .await
        }
        Err(e) => {
            eprintln!("Error generating audio: {}", e);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_music_video_path() {
        let id = "d7de2c63-aff6-4057-84eb-f273719f0a5f";
        let filename = format!("./tmp/music_videos/{}/video.mp4", id);
        let path = std::fs::canonicalize(&filename).unwrap();
        let full_path = path.into_os_string().into_string().unwrap();
        println!("Full Path: {}", full_path);
    }
}
