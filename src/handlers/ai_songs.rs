use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SunoResponse {
    pub id: String,
    pub video_url: String,
    pub audio_url: String,
    pub image_url: String,
    pub image_large_url: String,
    pub is_video_pending: bool,
    pub major_model_version: String,
    pub model_name: String,
    pub metadata: Metadata,
    pub display_name: String,
    pub handle: String,
    pub is_handle_updated: bool,
    pub avatar_image_url: String,
    pub is_following_creator: bool,
    pub user_id: String,
    pub created_at: String,
    pub status: String,
    pub title: String,
    pub play_count: i32,
    pub upvote_count: i32,
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub tags: String,
    pub prompt: String,
    pub gpt_description_prompt: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub duration: f64,
    pub refund_credits: bool,
    pub stream: bool,
}

#[async_trait]
impl EventHandler for AISongsHandler {
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
            match handle_requests(
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

#[derive(Default, Debug)]
struct AudioGenerationData {
    prompt: String,
    make_instrumental: bool,
    wait_audio: bool,
}

async fn generate_audio_by_prompt(
    data: AudioGenerationData,
) -> Result<serde_json::Value> {
    let base_url = "http://localhost:3000";
    let client = Client::new();
    let url = format!("{}/api/generate", base_url);

    // There must be a simpler way
    let payload = serde_json::json!({
        "prompt": data.prompt,
        "make_instrumental": data.make_instrumental,
        "wait_audio": data.wait_audio,
    });
    let response = client
        .post(&url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    Ok(response.json::<serde_json::Value>().await?)
}

pub async fn handle_requests(
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
    let prompt = splitmsg[1..].to_vec().join(" ");

    match command {
        "!song" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("It's Song time!");
            let data = AudioGenerationData {
                prompt: prompt,
                make_instrumental: false,
                wait_audio: true,
            };
            let res = generate_audio_by_prompt(data).await;
            match res {
                Ok(json_response) => {
                    println!("JSON Response: {:#?}", json_response);

                    let audio_url = &json_response["audio_url"];
                    let id = &json_response["id"];

                    // Now you can use suno_response
                    // println!("Generated audio: {}", audio_url.clone());
                    let file_name =
                        format!("ai_songs/{}.mp3", id);
                    let mut file = tokio::fs::File::create(&file_name).await?;
                    let url = &audio_url.to_string();
                    println!("URL: {}", url);
                    
                    let response =
                        reqwest::get(&audio_url.to_string()).await?;
                    let content = response.bytes().await?;
                    tokio::io::copy(&mut content.as_ref(), &mut file).await?;
                    println!("Downloaded audio to: {}", file_name);
                }
                Err(e) => {
                    eprintln!("Error generating audio: {}", e);
                }
            }
            // We have some text
            return Ok(());
        }

        _ => {
            return Ok(());
        }
    };
}
