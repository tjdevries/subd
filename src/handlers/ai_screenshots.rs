use crate::audio;
use crate::telephone;
use crate::dalle;
use crate::dalle::GenerateImage;
use crate::openai;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use subd_types::{Event, UserMessage};
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AiScreenshotsHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for AiScreenshotsHandler {
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
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // THEORY: We don't know if this is an explicit OBS message at this stage
            match handle_ai_screenshots(
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

pub async fn handle_ai_screenshots(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _is_mod = msg.roles.is_twitch_mod();
    let _is_vip = msg.roles.is_twitch_vip();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename = format!(
        "/home/begin/code/subd/tmp/screenshots/{}",
        unique_identifier
    );

    // I need a list of all models
    let models = vec!["dalle".to_string(), "sd".to_string()];
    let default_model = "sd".to_string();
    let model = splitmsg.get(1).unwrap_or(&default_model);

    let prompt = if splitmsg.len() > 1 {
        if models.contains(splitmsg.get(1).unwrap_or(&"".to_string())) {
            splitmsg[2..].to_vec().join(" ")
        } else {
            splitmsg[1..].to_vec().join(" ")
        }
    } else {
        "coolest programmer ever. Super stylish".to_string()
    };

    let model = model.clone();

    match command {
        "!new_alex" | "edit_alex" | "!ai_alex" => {
            let screenshot_source = "alex".to_string();
            let _ = screenshot_routing(
                sink,
                obs_client,
                filename,
                prompt,
                model,
                screenshot_source,
            )
            .await;
            return Ok(());
        }

        "!new_scene" | "edit_scene" | "!ai_scene" => {
            let screenshot_source = "Primary".to_string();
            let _ = screenshot_routing(
                sink,
                obs_client,
                filename,
                prompt,
                model,
                screenshot_source,
            )
            .await;
            return Ok(());
        }

        "!new_begin" | "edit_begin" | "!ai_begin" => {
            let screenshot_source = "begin".to_string();
            let _ = screenshot_routing(
                sink,
                obs_client,
                filename,
                prompt,
                model,
                screenshot_source,
            )
            .await;
            return Ok(());
        }

        _ => {
            return Ok(());
        }
    };
}

async fn screenshot_routing(
    sink: &Sink,
    obs_client: &OBSClient,
    filename: String,
    prompt: String,
    model: String,
    source: String,
) -> Result<()> {
    if model == "dalle" {
        let req = dalle::DalleRequest {
            prompt: prompt.clone(),
            username: "beginbot".to_string(),
            amount: 1,
        };
        let _ = telephone::create_screenshot_variation(
            sink, obs_client, filename, &req, prompt, source,
        )
        .await;
    } else {
        let req = dalle::StableDiffusionRequest {
            prompt: prompt.clone(),
            username: "beginbot".to_string(),
            amount: 1,
        };
        let _ = telephone::create_screenshot_variation(
            sink, obs_client, filename, &req, prompt, source,
        )
        .await;
    };

    Ok(())
}
