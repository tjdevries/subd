use ai_telephone;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use rand::seq::SliceRandom;
use rodio::*;
use std::thread;
use std::time;
use subd_openai::dalle;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AiScreenshotsTimerHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for AiScreenshotsTimerHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            match handle_ai_screenshots(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }

            // let t = time::Duration::from_millis(3000);
            let t = time::Duration::from_millis(1000);
            thread::sleep(t);
        }
    }
}

pub async fn handle_ai_screenshots(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    sink: &Sink,
) -> Result<String> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename =
        format!("../../tmp/screenshots/timelapse/{}", unique_identifier);

    let random_prompt = generate_random_prompt();
    let req = dalle::DalleRequest {
        prompt: random_prompt.clone(),
        username: "beginbot".to_string(),
        amount: 1,
    };
    ai_telephone::create_screenshot_variation(
        sink,
        obs_client,
        &filename,
        ai_telephone::ImageRequestType::Dalle(req),
        &random_prompt,
        "begin",
        Some("timelapse".to_string()),
    )
    .await
}

// This is key
pub fn generate_random_prompt() -> String {
    let choices = vec![
        "an 80's anime".to_string(),
        // "as a Pepe the frog".to_string(),
        // "album cover".to_string(),
        "as a ripped dude".to_string(),
        "as a crazy Bitcoin laser maxi".to_string(),
        "as a Ape".to_string(),
        "as SNES Pixel Art".to_string(),
        "as Modern Art Painting".to_string(),
        "anthropomorphic animals".to_string(),
        "rap album cover".to_string(),
        "outrun synthwave".to_string(),
        "propaganda poster".to_string(), // "newspaper".to_string(),
                                         // "fun".to_string(),
                                         // "beginbot as a service".to_string(),
                                         // "in a jail line up".to_string(),
                                         // "in an elon musk rocket ship on his way to mars".to_string(),
    ];
    let mut rng = rand::thread_rng();
    let selected_choice = choices.choose(&mut rng).unwrap_or_else(|| {
        eprintln!("Error: Failed to choose a random prompt");
        &choices[0] // Fallback to the first choice if selection fails
    });
    selected_choice.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_variation() {
        let _screenshot_prompt = generate_random_prompt();
        //assert_eq!(screenshot_prompt,"");
    }
}
