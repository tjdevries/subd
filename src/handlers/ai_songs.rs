use crate::audio::play_sound;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::Client;
use rodio::Decoder;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::thread;
use std::time;
use subd_types::{Event, UserMessage};
use twitch_chat::client::send_message;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use url::Url;

// 3. We create a `reqwest::Client` outside the loop to reuse it for better performance.
// 4. We use the `client.get(&cdn_url).send().await?` pattern instead of `reqwest::get` for consistency with the client usage.
pub struct AISongsHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

// We need to unify the suno responses
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

pub async fn handle_requests(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    sink: &Sink,
    twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
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

    match command {
        "!play" => {
            if _not_beginbot {
                return Ok(());
            }

            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => return Ok(()),
            };

            println!("\tQueuing {}", id);

            let file_name = format!("ai_songs/{}.mp3", id);
            let mp3 = match File::open(format!("{}", file_name)) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error opening sound file: {}", e);
                    return Ok(());
                }
            };

            let info = format!("@{} added {} to Queue", msg.user_name, id);
            let _ = send_message(&twitch_client, info).await;
            // Should this print a message to Twitch
            let file = BufReader::new(mp3);
            return play_sound_with_sink(sink, file).await;
        }

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        // ~~~~~ AUXILARY COMMANDS ~~~~ //
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~ //
        "!pause" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to play!");
            sink.pause();
            println!("\tDone Attempting to play!");
            return Ok(());
        }

        "!unpause" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to play!");
            sink.play();
            println!("\tDone Attempting to play!");
            return Ok(());
        }

        "!speedup" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tSpeeding up!");
            sink.set_speed(sink.speed() * 1.25);
            return Ok(());
        }

        "!up" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tTurning it Up!");
            sink.set_volume(sink.volume() * 1.20);
            return Ok(());
        }

        "!down" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tTurning it Down!");
            sink.set_volume(sink.volume() * 0.80);
            return Ok(());
        }

        "!normal" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tNormal Time");
            sink.set_speed(1.0);
            return Ok(());
        }

        "!slowdown" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tSlowin down!");
            sink.set_speed(sink.speed() * 0.75);
            return Ok(());
        }

        "!skip" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to Skip!");
            sink.skip_one();
            sink.play();
            println!("\tDone Attempting to Skip!");
            return Ok(());
        }

        "!stop" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to Stop!");
            sink.stop();
            println!("\tDone Attempting to Stop!");
            return Ok(());
        }

        // Reverb
        _ => {
            return Ok(());
        }
    }
}

// pub fn add_reverb(local_audio_path: String) -> Result<String> {
//     let audio_dest_path = add_postfix_to_filepath(
//         local_audio_path.clone(),
//         "_reverb".to_string(),
//     );
//     Command::new("sox")
//         .args(&[
//             "-t",
//             "wav",
//             &local_audio_path,
//             &audio_dest_path,
//             "gain",
//             "-2",
//             "reverb",
//             "70",
//             "100",
//             "50",
//             "100",
//             "10",
//             "2",
//         ])
//         .status()
//         .expect("Failed to execute sox");
//     Ok(audio_dest_path)
// }

async fn play_sound_with_sink(
    sink: &Sink,
    file: BufReader<File>,
) -> Result<()> {
    let sound = match Decoder::new(BufReader::new(file)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error decoding sound file: {}", e);
            return Err(anyhow!("Error decoding sound file: {}", e));
        }
    };
    sink.append(sound);

    // This could
    // This could be a message
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parsing_json() {
        let f = fs::read_to_string("tmp/raw_response_1725750380.json")
            .expect("Failed to open file");
        let suno_responses: Vec<SunoResponse> =
            serde_json::from_str(&f).expect("Failed to parse JSON");

        // let url = suno_responses[0].audio_url.as_str();
        // tokio::io::copy(&mut content.as_ref(), &mut file).await.unwrap();
        let id = &suno_responses[0].id;
        println!("Suno URL: {}", suno_responses[0].audio_url.as_str());

        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
        let file_name = format!("ai_songs/{}.mp3", id);

        let response = reqwest::get(cdn_url).await.unwrap();
        let mut file = tokio::fs::File::create(file_name).await.unwrap();
        let mut content = Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut file).unwrap();

        // assert!(!suno_responses.is_empty());
        // assert_eq!(suno_responses[0].status, "completed");
    }
}
