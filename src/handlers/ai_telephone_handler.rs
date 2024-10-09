use ai_images::image_generation;
use ai_telephone;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use stable_diffusion;
use subd_openai::dalle;
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
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // THEORY: We don't know if this is an explicit OBS message at this stage
            match handle_telephone_requests(
                &tx,
                &self.obs_client,
                &self.sink,
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
    obs_client: &OBSClient,
    sink: &Sink,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let command = splitmsg[0].as_str();
    let default = "".to_string();
    let image_url = splitmsg.get(1).unwrap_or(&default);
    let prompt = if splitmsg.len() > 1 {
        splitmsg[2..].to_vec().join(" ")
    } else {
        "".to_string()
    };

    match command {
        "!old_telephone" => {
            // we need to take an ID
            let id = splitmsg.get(1).unwrap();
            let _ = ai_telephone::old_obs_telephone_scene(obs_client, id).await;
            Ok(())
        }
        "!carlphone" => {
            let (_, unique_identifier) =
                image_generation::unique_archive_filepath(0, msg.user_name)?;
            let req = stable_diffusion::models::GenerateAndArchiveRequest {
                prompt: prompt.clone(),
                unique_identifier: unique_identifier.clone(),
                request_type: stable_diffusion::models::RequestType::Prompt2Img,
                set_as_obs_bg: true,
                additional_archive_dir: None,
                strength: None,
            };
            match ai_telephone::telephone(
                obs_client,
                sink,
                image_url,
                &prompt,
                5,
                &ai_telephone::ImageRequestType::StableDiffusion(req),
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error Carlphone Prompt: {}", e);
                    Ok(())
                }
            }
        }

        "!telephone" => {
            let req = dalle::DalleRequest {
                prompt: prompt.clone(),
                username: msg.user_name,
                amount: 1,
            };

            match ai_telephone::telephone(
                obs_client,
                sink,
                &image_url,
                &prompt,
                5,
                &ai_telephone::ImageRequestType::Dalle(req),
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error Telephone Prompt: {}", e);
                    Ok(())
                }
            }
        }

        _ => Ok(()),
    }
}
