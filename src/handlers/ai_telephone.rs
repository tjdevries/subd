use crate::openai;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use serde;
use subd_types::{Event, UserMessage};
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AiTelephoneHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for AiTelephoneHandler {
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
            match handle_telephone_requests(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
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

pub async fn handle_telephone_requests(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _is_mod = msg.roles.is_twitch_mod();
    let _is_vip = msg.roles.is_twitch_vip();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();

    match command {
        "!carlphone" => {
            let default = "".to_string();
            let image_url = splitmsg.get(1).unwrap_or(&default);
            let prompt = if splitmsg.len() > 1 {
                splitmsg[2..].to_vec().join(" ")
            } else {
                "".to_string()
            };

            println!("Telephone Prompt: {} ", prompt.clone());

            match openai::telephone2(image_url.to_string(), prompt, 5).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error Telephone Prompt: {}", e);
                    return Ok(());
                }
            }
        }

        "!telephone" => {
            // It shouldn't run if we don't have a URL
            let default = "".to_string();
            let image_url = splitmsg.get(1).unwrap_or(&default);
            // Crash if we don't have a prompt
            let prompt = splitmsg[2..].to_vec().join(" ");

            println!("Telephone Prompt: {} ", prompt.clone());
            match openai::telephone(image_url.to_string(), prompt, 5).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error Telephone Prompt: {}", e);
                    return Ok(());
                }
            }
        }

        _ => {
            return Ok(());
        }
    };
}