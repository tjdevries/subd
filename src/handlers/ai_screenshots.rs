use crate::openai;
use crate::dalle;
use crate::dalle::GenerateImage;
use std::time;
use std::fs::File;
use std::io::BufReader;
use chrono::Utc;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use serde;
use std::thread;
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

// This also has a pause in it,
// we might want to take that in as a variable
async fn play_sound(sink: &Sink) -> Result<()> {
    let file = BufReader::new(
        File::open(format!("./MP3s/{}.mp3", "aim")).unwrap(),
    );
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);
    // To tell me a screen shot is coming
    sink.set_volume(0.3);
    sink.append(Decoder::new(BufReader::new(file)).unwrap());
    sink.sleep_until_end();
    Ok(())
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
    let prompt = if splitmsg.len() > 1 {
        splitmsg[1..].to_vec().join(" ")
    } else {
        "coolest programmer ever".to_string()
    };

    match command {
        "!edit_scene" => {
            let _ = play_sound(&sink).await;

            let _ = openai::save_screenshot(&obs_client, "Primary", &filename)
                .await;

            let description =
                openai::ask_gpt_vision2(&filename, None).await.unwrap();

            let new_description = format!(
                "{} {} . The most important thing to focus on is: {}",
                prompt, description, prompt
            );

            println!("Generating Dalle Image: {}", new_description.clone());

            let req = dalle::DalleRequest {
                prompt: new_description,
                username: "beginbot".to_string(),
                amount: 1,
            };

            let dalle_path = req.generate_image().await;

            println!("Dalle Path: {}", dalle_path);

            return Ok(())
        }

        "!new_begin2" => {
            let _ = play_sound(&sink).await;

            let _ =
                openai::save_screenshot(&obs_client, "begin", &filename).await;

            let description =
                openai::ask_gpt_vision2(&filename, None).await.unwrap();

            let new_description = format!(
                "{} {} . The most important thing to focus on is: {}",
                prompt, description, prompt
            );

            println!("Generating Dalle Image: {}", new_description.clone());

            let stable_diffusion_req = dalle::StableDiffusionRequest{
                prompt: new_description,
                username: "beginbot".to_string(),
            };

            let dalle_path = stable_diffusion_req.generate_image().await;

            println!("Dalle Path: {}", dalle_path);

            return Ok(());
        }

        "!new_begin" => {
            let _ = play_sound(&sink).await;
            
            let _ =
                openai::save_screenshot(&obs_client, "begin", &filename).await;

            let description =
                openai::ask_gpt_vision2(&filename, None).await.unwrap();

            let new_description = format!(
                "{} {} . The most important thing to focus on is: {}",
                prompt, description, prompt
            );

            println!("Generating Dalle Image: {}", new_description.clone());

            let req = dalle::DalleRequest{
                prompt: new_description,
                username: "beginbot".to_string(),
                amount: 1,
            };

            let dalle_path =
                req.generate_image().await;

            println!("Dalle Path: {}", dalle_path);

            return Ok(());
        }
        _ => {
            return Ok(());
        }
    };
}
