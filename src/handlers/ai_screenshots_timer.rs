use crate::dalle;
use crate::dalle::GenerateImage;
use crate::openai;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use rand::seq::SliceRandom;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time;
use subd_types::{Event, UserMessage};
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

            let t = time::Duration::from_millis(3000);
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

    let filename = format!(
        "/home/begin/code/subd/tmp/screenshots/timelapse/{}",
        unique_identifier
    );

    let random_prompt = generate_random_prompt();
    let req = dalle::DalleRequest {
        prompt: random_prompt.clone(),
        username: "beginbot".to_string(),
        amount: 1,
    };
    create_screenshot_variation(
        sink,
        obs_client,
        filename,
        &req,
        random_prompt,
        "begin".to_string(),
    )
    .await

    // We need to save this!
}

// This is key
pub fn generate_random_prompt() -> String {
    let choices = vec![
        "an 80's anime".to_string(),
        "as a Pepe the frog".to_string(),
        "album cover".to_string(),
        "newspaper".to_string(),
        "fun".to_string(),
        "beginbot as a service".to_string(),
        "in a jail line up".to_string(),
        "in an elon musk rocket ship on his way to mars".to_string(),
    ];
    let mut rng = rand::thread_rng();
    let selected_choice = choices.choose(&mut rng).unwrap();
    selected_choice.to_string()
}

// TODO: I don't like the name
async fn create_screenshot_variation(
    sink: &Sink,
    obs_client: &OBSClient,
    filename: String,
    ai_image_req: &impl GenerateImage,
    prompt: String,
    source: String,
) -> Result<String> {
    // let _ = play_sound(&sink).await;

    let _ = openai::save_screenshot(&obs_client, &source, &filename).await;

    let description = openai::ask_gpt_vision2(&filename, None).await.unwrap();

    let new_description = format!(
        "{} {} . The most important thing to focus on is: {}",
        prompt, description, prompt
    );
    // ai_image_req.prompt = new_description;

    // println!("Generating Dalle Image: {}", new_description.clone());

    let dalle_path = ai_image_req
        .generate_image(new_description, Some("timelapse".to_string()), false)
        .await;

    println!("Dalle Path: {}", dalle_path);

    Ok(dalle_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_variation() {
        let screenshot_prompt = generate_random_prompt();
        //assert_eq!(screenshot_prompt,"");
    }
}
