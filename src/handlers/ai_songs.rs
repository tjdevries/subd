use anyhow::anyhow;
use std::fs::File;
use std::thread;
use rodio::Decoder;
use std::io::BufReader;
use std::io::Cursor;
use std::time;
use std::fs;
use url::Url;
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SunoResponse {
    pub id: String,
    pub video_url: String,
    pub audio_url: String,
    pub image_url: String,
    pub image_large_url: Option<String>,
    pub is_video_pending: Option<bool>,

    #[serde(default)]
    pub major_model_version: String,
    pub model_name: String,

    #[serde(default)]
    pub metadata: Metadata,
    
    #[serde(default)]
    pub display_name: String,

    #[serde(default)]
    pub handle: String,
    #[serde(default)]
    pub is_handle_updated: bool,
    #[serde(default)]
    pub avatar_image_url: String,
    #[serde(default)]
    pub is_following_creator: bool,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub play_count: i32,
    #[serde(default)]
    pub upvote_count: i32,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
        let raw_json = response.text().await?;
        let tmp_file_path = format!("tmp/suno_responses/{}.json", chrono::Utc::now().timestamp());
        tokio::fs::write(&tmp_file_path, &raw_json).await?;
        println!("Raw JSON saved to: {}", tmp_file_path);
        Ok(serde_json::from_str::<serde_json::Value>(&raw_json)?)
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
    
    let is_mod = msg.roles.is_twitch_mod();
    let is_vip = msg.roles.is_twitch_vip();
    let is_sub = msg.roles.is_twitch_sub();

    let command = splitmsg[0].as_str();
    let prompt = splitmsg[1..].to_vec().join(" ");

    match command {
        "!song" => {
            if !is_sub && !is_vip && !is_mod && _not_beginbot {
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

                    // TODO: download both songs
                    // Use status maybe eventually
                    let status = &json_response[0]["status"];
                    let id = &json_response[0]["id"];
                    
                    let tmp_file_path = format!("tmp/suno_responses/{}.json", id.as_str().unwrap());
                    tokio::fs::write(&tmp_file_path, &json_response.to_string()).await?;

                    let file_name =
                        format!("ai_songs/{}.mp3", id.as_str().unwrap());
                    let mut file = tokio::fs::File::create(&file_name).await?;

                    let mut response;
                    loop {
                        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id.as_str().unwrap());
                        println!("Attempting to Download song at: {}", cdn_url);
                        response = reqwest::get(&cdn_url).await?;
                        if response.status().is_success() {
                            let content = response.bytes().await?;
                            tokio::io::copy(&mut content.as_ref(), &mut file).await?;
                            println!("Downloaded audio to: {}", file_name);
                            let mp3 = match File::open(format!("{}", file_name))
                            {
                                Ok(v) => v,
                                Err(e) => {
                                    eprintln!("Error opening sound file: {}", e);
                                    continue;
                                }
                            };

                            
                            let file = BufReader::new(mp3);
                            sink.set_volume(0.1);
                            let sound = match Decoder::new(BufReader::new(file)) {
                                Ok(v) => v,
                                Err(e) => {
                                    eprintln!("Error decoding sound file: {}", e);
                                    continue;
                                }
                            };

                            sink.append(sound);
                            sink.sleep_until_end();
                            let sleep_time = time::Duration::from_millis(100);
                            thread::sleep(sleep_time);
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                    
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_parsing_json() {
        let f = fs::read_to_string("tmp/raw_response_1725750380.json").expect("Failed to open file");
        let suno_responses: Vec<SunoResponse> = serde_json::from_str(&f).expect("Failed to parse JSON");

        // let url = suno_responses[0].audio_url.as_str();
        // tokio::io::copy(&mut content.as_ref(), &mut file).await.unwrap();
        let id = &suno_responses[0].id;
        println!("Suno URL: {}", suno_responses[0].audio_url.as_str());
        
        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
        let file_name = format!("ai_songs/{}.mp3", id);
        
        let response = reqwest::get(cdn_url).await.unwrap();
        let mut file = std::fs::File::create(file_name).unwrap();
        let mut content =  Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();

        // assert!(!suno_responses.is_empty());
        // assert_eq!(suno_responses[0].status, "completed");
    }
}
