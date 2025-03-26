use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use sqlx::PgPool;
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
    song_id: &str,
    name: &str,
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
    match parse_command(&msg, pool).await? {
        Command::Unknown => Ok(()),
        Command::CreateMusicVideoVideo { id, image_name } => {
            let result = find_image_filename(&id, &image_name).await;
            match result {
                Ok((image_filename, path_string, _path)) => {
                    let id_clone = id.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            ai_music_videos::create_video_from_image(
                                &id_clone,
                                &image_filename,
                                &path_string,
                            )
                            .await
                        {
                            eprintln!("Error creating video from image: {}", e);
                        }
                    });
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

            let scene = "AIFriends";
            let source = "music-video";
            obs_service::obs::update_obs_source(
                obs_client, &filename, scene, source,
            )
            .await
        }
        Command::CreateMusicVideoImages { id, count } => {
            for index in 0..count {
                println!("Creating Image for Index: {}", index);
                // Do we need sleep a little before calling this so fast again??
                let pool_clone = pool.clone();
                let id_clone = id.clone();
                tokio::spawn(async move {
                    let res = ai_music_videos::create_music_video_image(
                        &pool_clone,
                        id_clone,
                        None,
                        Some(index + 1),
                    )
                    .await;
                    if let Err(e) = res {
                        eprintln!("Error creating music video image: {}", e);
                    }
                });
            }
            Ok(())
        }
        Command::CreateMusicVideoImage { id, prompt } => {
            let _res = ai_music_videos::create_music_video_image(
                pool, id, prompt, None,
            )
            .await;
            Ok(())
        }
    }
}

/// Parses a user's message into a `Command`.
async fn parse_command(msg: &UserMessage, pool: &PgPool) -> Result<Command> {
    let mut words = msg.contents.split_whitespace();
    match words.next() {
        Some("!create_music_video_images") | Some("!generate_images") => {
            let id = ai_playlist::get_current_song(pool)
                .await?
                .song_id
                .to_string();
            let count = msg
                .contents
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(1);

            println!("Generating Images: {} - Count: {}", id, count);
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
            // We need an async move
            let current_song = ai_playlist::get_current_song(pool).await?;
            Ok(Command::CreateMusicVideoVideo {
                id: current_song.song_id.to_string(),
                image_name,
            })
        }

        Some("!generate_image") => {
            let current_song = ai_playlist::get_current_song(pool).await?;
            println!("Generating Image for Song: {}", current_song.song_id);

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
