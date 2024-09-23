use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use fal_ai;
use obws::Client as OBSClient;
use rodio::*;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

pub struct FalHandler {
    // pub queue_rx: &'static queue::SourcesQueueOutput<f32>,
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for FalHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match handle_fal_commands(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}

pub async fn handle_fal_commands(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    //let default_source = constants::DEFAULT_SOURCE.to_string();
    // let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let is_mod = msg.roles.is_twitch_mod();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let command = splitmsg[0].as_str();

    match command {
        // This sets the theme, that is passed to the image generation prompt
        "!theme" => {
            if _not_beginbot && !is_mod {
                return Ok(());
            }
            let theme = &splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");
            twitch_stream_state::set_ai_background_theme(pool, theme).await?;
        }

        "!talk" => {
            // Not sure why this is hardcoded here
            println!("\n\nTALK TIME!");
            let image_file_path = "teej_2.jpg";
            let _ = fal_ai::create_video_from_image(image_file_path).await;
        }

        _ => {
            // TODO: a way to enable or disable Fal on every chat-message
            let word_count = msg.contents.split_whitespace().count();
            if !command.starts_with('!')
                && !command.starts_with('@')
                && word_count > 1
            {
                let prompt = msg.contents;
                let theme =
                    twitch_stream_state::get_ai_background_theme(pool).await?;
                let final_prompt = format!("{} {}", theme, prompt);
                println!("Creating image for prompt: {}", final_prompt);
                fal_ai::create_turbo_image(final_prompt).await?;
            }
        }
    };

    Ok(())
}
