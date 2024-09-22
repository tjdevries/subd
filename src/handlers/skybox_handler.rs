use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use serde;
use serde::{Deserialize, Serialize};
use skybox;
use skybox::check_skybox_status_and_save;
use subd_types::Event;
use subd_types::UserMessage;
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub enum ChatArgPosition {
    StyleID(i32),
    Prompt(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkyboxStyle {
    pub id: i32,
    pub name: String,
    pub max_char: String,
    pub image: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratorData {
    pub prompt: String,
    pub negative_text: String,
    pub animation_mode: String,
}

pub struct SkyboxHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
}

pub struct SkyboxRoutingHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let request = match event {
                Event::SkyboxRequest(msg) => msg,
                _ => continue,
            };

            println!("Attempting to Skybox");
            match skybox::request_skybox(
                self.pool.clone(),
                request.msg,
                request.style_id,
            )
            .await
            {
                Ok(v) => {}
                Err(e) => continue,
            };
        }
    }
}

#[async_trait]
impl EventHandler for SkyboxRoutingHandler {
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

            match handle_skybox_commands(
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

pub async fn handle_skybox_commands(
    tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    _pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<(), String> {
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();

    match command {
        // ===========================================
        // == Skybox
        // ===========================================
        "!previous" => {
            let default_skybox_id = String::from("2449796");
            let skybox_id: &str = splitmsg.get(1).unwrap_or(&default_skybox_id);
            let file_path =
                "/home/begin/code/BeginGPT/tmp/current/previous.txt";
            if let Err(e) = skybox::write_to_file(file_path, skybox_id) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to Return to previous Skybox! {}", skybox_id);
            Ok(())
        }

        // This needs to take an ID
        "!skybox_styles" => {
            let styles = skybox::styles_for_chat().await;
            println!("\n\nStyles Time: {:?}", styles);

            // So we think this code isn't returning all chunks
            let chunks = chunk_string(&styles, 500);
            for chunk in chunks {
                println!("Chunk: {}", chunk);
                twitch_chat::client::send_message(twitch_client, chunk)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }

        "!check_skybox" => {
            if not_beginbot {
                return Ok(());
            }

            // obs_client
            let _ = check_skybox_status_and_save(9612607).await;
            Ok(())
        }

        // We need to eventually take in style IDs
        "!skybox" => {
            // if not_beginbot {
            //     return Ok(());
            // }
            if let Ok(style_id) = find_style_id(splitmsg.clone()) {
                println!("\tStyle ID: {}", style_id);

                let skybox_info = if style_id == 1 {
                    splitmsg
                        .clone()
                        .into_iter()
                        .skip(1)
                        .collect::<Vec<String>>()
                        .join(" ")
                } else {
                    splitmsg
                        .clone()
                        .into_iter()
                        .skip(2)
                        .collect::<Vec<String>>()
                        .join(" ")
                };

                println!("Sending Skybox Request: {}", skybox_info.clone());
                let _ =
                    tx.send(Event::SkyboxRequest(subd_types::SkyboxRequest {
                        msg: skybox_info,
                        style_id,
                    }));
            }

            Ok(())
        }

        "!remix" => {
            let remix_info = splitmsg
                .clone()
                .into_iter()
                .skip(1)
                .collect::<Vec<String>>()
                .join(" ");
            let file_path = "/home/begin/code/BeginGPT/tmp/current/remix.txt";
            if let Err(e) = skybox::write_to_file(file_path, &remix_info) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to  Remix! {}", remix_info);
            Ok(())
        }

        _ => Ok(()),
    }
}

// People need to supply their own default if none is returned
// let default_style_id = 1;
pub fn find_style_id(splitmsg: Vec<String>) -> Result<i32, String> {
    let range = 1..=47;
    // we need to check the range

    splitmsg
        .get(1)
        .ok_or("No Style ID Arg found".to_string())
        .map(|m| m.parse::<i32>())?
        .map_err(|e| e.to_string())
        .and_then(|m| {
            range
                .contains(&m)
                .then_some(m)
                .ok_or("Error finding Style ID".to_string())
        })
}

// Where does this belong
// Also it doesn't work
pub fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut last_split = 0;
    let mut current_count = 0;

    for (idx, ch) in s.char_indices() {
        current_count += 1;

        if (ch.to_string() == "," || idx == s.len() - 1)
            && current_count >= chunk_size
        {
            chunks.push(s[last_split..=idx].to_string());

            last_split = idx + 1;
            current_count = 0;
        }
    }

    if last_split < s.len() {
        chunks.push(s[last_split..].to_string());
    }

    chunks
}
