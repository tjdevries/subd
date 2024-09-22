use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use sqlx::PgPool;
use subd_types::Event;
use subd_types::UserMessage;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsDownloader {
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for AISongsDownloader {
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

            match handle_requests(
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

pub async fn handle_requests(
    tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    // let is_mod = msg.roles.is_twitch_mod();
    // let is_vip = msg.roles.is_twitch_vip();
    // let is_sub = msg.roles.is_twitch_sub();
    // if !is_sub && !is_vip && !is_mod && _not_beginbot {
    //     return Ok(());
    // }

    let command = splitmsg[0].as_str();
    let prompt = splitmsg[1..].to_vec().join(" ");

    match command {
        "!download" => {
            if _not_beginbot {
                return Ok(());
            }

            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => return Ok(()),
            };

            subd_suno::download_and_play(
                twitch_client,
                tx,
                msg.user_name,
                &id.to_string(),
            )
            .await
        }

        "!create_song" | "!song" => {
            println!("\tIt's Song time!");
            let data = subd_suno::AudioGenerationData {
                prompt,
                make_instrumental: false,
                wait_audio: true,
            };
            let res = subd_suno::generate_audio_by_prompt(data).await;
            match res {
                Ok(json_response) => {
                    // There is a better way of doing this
                    println!("JSON Response: {:#?}", json_response);
                    let _ = subd_suno::parse_suno_response_download_and_play(
                        twitch_client,
                        pool,
                        tx,
                        json_response.clone(),
                        0,
                        msg.user_name.clone(),
                    )
                    .await;
                    subd_suno::parse_suno_response_download_and_play(
                        twitch_client,
                        pool,
                        tx,
                        json_response,
                        1,
                        msg.user_name.clone(),
                    )
                    .await
                }
                Err(e) => {
                    eprintln!("Error generating audio: {}", e);
                    Ok(())
                }
            }
        }

        _ => {
            Ok(())
        }
    }
}
