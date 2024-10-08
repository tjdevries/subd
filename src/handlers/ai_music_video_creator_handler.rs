use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use sqlx::PgPool;
use std::sync::Arc;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AIMusicVideoCreatorHandler {
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

enum Command {
    CreateMusicVideoVideo { id: String, image_name: String },
    CreateMusicVideoImage { id: String, prompt: Option<String> },
    CreateMusicVideoImages { id: String, count: i64 },
    CreateMusicVideo { id: String },
    Unknown,
}

#[async_trait]
impl EventHandler for AIMusicVideoCreatorHandler {
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

async fn find_image_filename(
    song_id: String,
    name: String,
) -> Result<(String, String, std::path::PathBuf)> {
    println!("Finding Image for Filename: {}", name);
    let dir_path = format!("./tmp/music_videos/{}/", song_id);
    let entries = std::fs::read_dir(&dir_path)
        .map_err(|_| anyhow!("Failed to read directory: {}", dir_path))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Failed to get file extension"))?;

        if !["png", "jpeg", "jpg"].contains(&extension) {
            continue;
        }

        let file_stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow!("Failed to get file stem"))?;

        if file_stem == name {
            let p = path
                .to_str()
                .ok_or_else(|| anyhow!("Failed to convert path to string"))
                .map(String::from)?;
            return Ok((p, path.to_string_lossy().into_owned(), path));
        }
    }

    Err(anyhow!("No matching image found for: {}", name))
}

/// Handles incoming requests based on the parsed command.
pub async fn handle_requests(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    msg: UserMessage,
) -> Result<()> {
    // Ignore messages from the bot itself
    if ["nightbot"].contains(&msg.user_name.as_str()) {
        return Ok(());
    }

    let _song_id = ai_playlist::get_current_song(pool)
        .await?
        .song_id
        .to_string();
    // These are named wrong right now
    match parse_command(&msg, pool).await? {
        Command::Unknown => Ok(()),
        Command::CreateMusicVideoVideo { id, image_name } => {
            let result = find_image_filename(id.clone(), image_name).await;
            match result {
                Ok((image_filename, path_string, _path)) => {
                    ai_music_videos::create_video_from_image(
                        &id,
                        &image_filename,
                        &path_string,
                    )
                    .await?;
                }
                Err(e) => {
                    let _ = send_message(
                        twitch_client,
                        format!(
                            "Error finding Image to create Video from: {}",
                            e
                        ),
                    )
                    .await;
                }
            };
            Ok(())
        }
        Command::CreateMusicVideo { id } => {
            let filename =
                ai_music_videos::create_music_video_images_and_video(pool, id)
                    .await?;
            update_obs_source(obs_client, &filename).await
        }
        Command::CreateMusicVideoImages { id, count } => {
            for _ in 0..count {
                let pool_clone = pool.clone();
                let id_clone = id.clone();
                tokio::spawn(async move {
                    let _ = ai_music_videos::create_music_video_images(
                        &pool_clone,
                        id_clone,
                    )
                    .await;
                });
            }
            Ok(())
        }
        Command::CreateMusicVideoImage { id, prompt } => {
            let _res =
                ai_music_videos::create_music_video_image(pool, id, prompt)
                    .await;
            Ok(())
        }
    }
}

async fn update_obs_source(
    obs_client: &OBSClient,
    filename: &str,
) -> Result<()> {
    let path = std::fs::canonicalize(filename)?;
    let full_path = path
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("Failed to convert path to string"))?;

    let source = "music-video".to_string();
    let _ = obs_service::obs_source::set_enabled(
        "AIFriends",
        &source,
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

    obs_service::obs_scenes::change_scene(obs_client, "Movie Trailer").await
}

/// Parses a user's message into a `Command`.
async fn parse_command(msg: &UserMessage, pool: &PgPool) -> Result<Command> {
    let mut words = msg.contents.split_whitespace();
    match words.next() {
        Some("!create_music_video_images") | Some("!generate_images") => {
            let id = match words.next() {
                Some(id) => id.to_string(),
                None => ai_playlist::get_current_song(pool)
                    .await?
                    .song_id
                    .to_string(),
            };
            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let count = if splitmsg.len() > 1 {
                splitmsg[1].parse::<i64>().unwrap_or(1)
            } else {
                1
            };
            Ok(Command::CreateMusicVideoImages { id, count })
        }

        Some("!generate_video") => {
            let image_name = match words.next() {
                Some(name) => name.to_string(),
                None => {
                    return Err(anyhow!(
                        "No image name provided for video generation"
                    ))
                }
            };
            let current_song = ai_playlist::get_current_song(pool).await?;
            Ok(Command::CreateMusicVideoVideo {
                id: current_song.song_id.to_string(),
                image_name,
            })
        }

        Some("!generate_image") => {
            let current_song = ai_playlist::get_current_song(pool).await?;

            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let prompt = if splitmsg.len() > 1 {
                Some(splitmsg[1..].join(" "))
            } else {
                None
            };
            Ok(Command::CreateMusicVideoImage {
                id: current_song.song_id.to_string(),
                prompt,
            })
        }
        Some("!create_music_video") => {
            let id = match words.next() {
                Some(id) => id.to_string(),
                None => ai_playlist::get_current_song(pool)
                    .await?
                    .song_id
                    .to_string(),
            };
            Ok(Command::CreateMusicVideo { id })
        }
        _ => Ok(Command::Unknown),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_music_video_path() {
        println!();
    }
}
