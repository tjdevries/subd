use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use sqlx::PgPool;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsDownloader {
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

enum Command {
    Download { id: String },
    CreateSong { prompt: String },
    CreateInstrumentalSong { prompt: String },
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
                if let Err(err) =
                    handle_requests(&tx, &self.twitch_client, &self.pool, msg)
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
            handle_download_command(
                pool,
                twitch_client,
                tx,
                &msg.user_name,
                &id,
            )
            .await
        }
        Command::CreateSong { prompt } => {
            handle_create_song_command(
                twitch_client,
                pool,
                tx,
                &msg.user_name,
                &prompt,
                false,
            )
            .await
        }
        Command::CreateInstrumentalSong { prompt } => {
            handle_create_song_command(
                twitch_client,
                pool,
                tx,
                &msg.user_name,
                &prompt,
                true,
            )
            .await
        }
        Command::Unknown => Ok(()),
    }
}

/// Parses a user's message into a `Command`.
fn parse_command(msg: &UserMessage) -> Command {
    let mut words = msg.contents.split_whitespace();
    match words.next() {
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
        Some("!create_instrumental_song") | Some("!instrumental") => {
            let prompt = words.collect::<Vec<_>>().join(" ");
            Command::CreateInstrumentalSong { prompt }
        }
        _ => Command::Unknown,
    }
}

async fn handle_download_command(
    pool: &PgPool,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    tx: &broadcast::Sender<Event>,
    username: &str,
    id: &str,
) -> Result<()> {
    subd_suno::download_and_play(
        pool,
        twitch_client,
        tx,
        username,
        id.to_string(),
    )
    .await
}

async fn handle_create_song_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    tx: &broadcast::Sender<Event>,
    user_name: &str,
    prompt: &str,
    instrumental: bool,
) -> Result<()> {
    println!("\tIt's Song time!");
    let data = subd_suno::AudioGenerationData {
        prompt: prompt.to_string(),
        make_instrumental: instrumental,
        wait_audio: true,
    };
    match subd_suno::generate_audio_by_prompt(data).await {
        Ok(json_response) => {
            println!("JSON Response: {:#?}", json_response);
            subd_suno::parse_suno_response_download_and_play(
                twitch_client,
                pool,
                tx,
                prompt,
                json_response.clone(),
                0,
                user_name,
            )
            .await?;
            subd_suno::parse_suno_response_download_and_play(
                twitch_client,
                pool,
                tx,
                prompt,
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

    #[test]
    fn test_music_video_path() {
        let id = "d7de2c63-aff6-4057-84eb-f273719f0a5f";
        let filename = format!("./tmp/music_videos/{}/video.mp4", id);
        let path = std::fs::canonicalize(filename).unwrap();
        let full_path = path.into_os_string().into_string().unwrap();
        println!("Full Path: {}", full_path);
    }
}
