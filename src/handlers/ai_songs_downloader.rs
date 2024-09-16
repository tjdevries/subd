use crate::audio::play_sound;
use chrono::Utc;
use uuid::{uuid, Uuid};
use crate::ai_song_playlist::ai_song_playlist::Model;
// use crate::ai_song_playlist::ai_song_playlist::Model;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
// use rodio::Decoder;
// use rodio::Sink;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::thread;
use std::time;
use subd_types::{Event, UserMessage};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use url::Url;

use super::fal_handler;

// 3. We create a `reqwest::Client` outside the loop to reuse it for better performance.
// 4. We use the `client.get(&cdn_url).send().await?` pattern instead of `reqwest::get` for consistency with the client usage.
pub struct AISongsDownloader {
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

            // THEORY: We don't know if this is an explicit OBS message at this stage
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

#[derive(Default, Debug)]
struct AudioGenerationData {
    prompt: String,
    make_instrumental: bool,
    wait_audio: bool,
}

pub async fn handle_requests(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
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
            return download_and_play(
                twitch_client,
                tx,
                msg.user_name,
                &id.to_string(),
            )
            .await;
        }

        "!create_song" | "!song" => {
            println!("\tIt's Song time!");
            let data = AudioGenerationData {
                prompt,
                make_instrumental: false,
                wait_audio: true,
            };
            let res = generate_audio_by_prompt(data).await;
            match res {
                Ok(json_response) => {
                    // There is a better way of doing this
                    println!("JSON Response: {:#?}", json_response);
                    let _ = parse_suno_response_download_and_play(
                        twitch_client,
                        pool,
                        tx,
                        json_response.clone(),
                        0,
                        msg.user_name.clone(),
                    )
                    .await;
                    parse_suno_response_download_and_play(
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
                    return Ok(());
                }
            }
        }

        _ => {
            return Ok(());
        }
    }
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
    let tmp_file_path =
        format!("tmp/suno_responses/{}.json", chrono::Utc::now().timestamp());
    tokio::fs::write(&tmp_file_path, &raw_json).await?;
    println!("Raw JSON saved to: {}", tmp_file_path);
    Ok(serde_json::from_str::<serde_json::Value>(&raw_json)?)
}

async fn download_and_play(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    tx: &broadcast::Sender<Event>,
    user_name: String,
    id: &String,
) -> Result<()> {
    let id = id.clone();
    let tx = tx.clone();
    let twitch_client = twitch_client.clone();

    thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id.as_str());
            loop {
                println!(
                    "{} | Attempting to Download song at: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    cdn_url
                );
                let response = reqwest::get(&cdn_url).await.unwrap();

                if response.status().is_success() {
                    let _file = just_download(response, id.to_string()).await;

                    let info = format!("@{}'s song {} added to the Queue. Begin needs to !skip to get to your position faster", user_name, id.to_string());

                    let _ =
                        tx.send(Event::UserMessage(subd_types::UserMessage {
                            user_name: "beginbot".to_string(),
                            contents: format!("!play {}", id.to_string()),
                            ..Default::default() // user_name: msg.user_name,
                        }));

                    break;
                }

                // Sleep for 5 seconds before trying again
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        })
    });
    return Ok(());
}

async fn parse_suno_response_download_and_play(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    tx: &broadcast::Sender<Event>,
    json_response: serde_json::Value,
    index: usize,
    user_name: String,
) -> Result<()> {

    let id = json_response[index]["id"].as_str().unwrap();
    let song_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!(e.to_string()) )?;
    let lyrics = &json_response[index]["lyric"].as_str().unwrap();
    let title = &json_response[index]["title"].as_str().unwrap();
    let prompt = &json_response[index]["prompt"].as_str().unwrap();
    let tags = &json_response[index]["tags"].as_str().unwrap();
    let audio_url = &json_response[index]["audio_url"].as_str().unwrap();
    let gpt_description_prompt = &json_response[index]["gpt_description_prompt"].as_str().unwrap();
    let created_at =
        sqlx::types::time::OffsetDateTime::now_utc();
    let new_song = Model {
        song_id: song_id,
        title: title.to_string(),
        tags: tags.to_string(),
        prompt: prompt.to_string(),
        username: user_name.clone(),
        audio_url: audio_url.to_string(),
        lyric: lyrics.to_string(),
        gpt_description_prompt: gpt_description_prompt.to_string(),
        last_updated: Some(created_at),
        created_at: Some(created_at),
    };
    let saved_song = new_song.save(&pool).await?;
    println!("{:?}", saved_song);
    
    let lyric_lines: Vec<&str> = lyrics.split('\n').collect();

    let folder_path = format!("tmp/suno_responses/{}", id);
    tokio::fs::create_dir_all(&folder_path).await?;

    tokio::fs::write(
        format!("tmp/suno_responses/{}.json", id),
        &json_response.to_string(),
    )
    .await?;

    // This needs to happen async and work properly
    // for (_i, line) in lyric_lines.iter().enumerate() {
    //     fal_handler::create_turbo_image_in_folder(
    //         line.to_string(),
    //         &folder_path,
    //     )
    //     .await?;
    // }

    download_and_play(twitch_client, tx, user_name, &id.to_string()).await
}

// We should return the file and have it played somewhere else
async fn just_download(
    response: reqwest::Response,
    id: String,
) -> Result<BufReader<File>> {
    let file_name = format!("ai_songs/{}.mp3", id);
    let mut file = tokio::fs::File::create(&file_name).await?;

    let content = response.bytes().await?;
    tokio::io::copy(&mut content.as_ref(), &mut file).await?;
    println!("Downloaded audio to: {}", file_name);
    let mp3 = match File::open(format!("{}", file_name)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error opening sound file: {}", e);
            return Err(anyhow!("Error opening sound file: {}", e));
        }
    };
    let file = BufReader::new(mp3);

    return Ok(file);
}
