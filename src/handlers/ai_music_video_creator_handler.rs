use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use sqlx::PgPool;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
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

/// Handles incoming requests based on the parsed command.
pub async fn handle_requests(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &PgPool,
    msg: UserMessage,
) -> Result<()> {
    // Ignore messages from the bot itself
    if ["nightbot"].contains(&msg.user_name.as_str()) {
        return Ok(());
    }

    match parse_command(&msg, pool).await? {
        Command::Unknown => Ok(()),
        Command::CreateMusicVideo { id } => {
            Ok(handle_create_music_video(obs_client, pool, id).await?)
        }
    }
}

async fn handle_create_music_video(
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    id: String,
) -> Result<()> {
    let filename = ai_music_videos::create_music_video_2(pool, id).await?;
    let path = std::fs::canonicalize(&filename)?;
    let full_path = path
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("Failed to convert path to string"))?;

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

    obs_service::obs_scenes::change_scene(obs_client, "Movie Trailer").await
}

/// Parses a user's message into a `Command`.
async fn parse_command(msg: &UserMessage, pool: &PgPool) -> Result<Command> {
    let mut words = msg.contents.split_whitespace();
    match words.next() {
        Some("!create_music_video") => {
            if let Some(id) = words.next() {
                Ok(Command::CreateMusicVideo { id: id.to_string() })
            } else {
                let current_song = ai_playlist::get_current_song(pool).await?;
                Ok(Command::CreateMusicVideo {
                    id: current_song.song_id.to_string(),
                })
            }
        }
        _ => Ok(Command::Unknown),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_music_video_path() {
        println!("");
    }
}
