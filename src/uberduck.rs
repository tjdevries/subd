use crate::obs;
use crate::obs_text;
use crate::stream_character;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use events::EventHandler;
// use std::rand::{task_rng, Rng};

// use rand::thread_rng;
use rodio::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufWriter, Write};
// use std::rand;
// use std::rand::Rng;
use std::{thread, time};
use subd_types::Event;
use subd_types::SourceVisibilityRequest;
use subd_types::StreamCharacterRequest;
use subd_types::TransformOBSTextRequest;
use subd_types::UberDuckRequest;
use tokio::sync::broadcast;

pub struct UberDuckHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}

pub struct ExpertUberDuckHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}

// If we parse the full list this is all we'll use
#[derive(Serialize, Deserialize, Debug)]
struct UberDuckVoice {
    category: String,
    display_name: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckVoiceResponse {
    uuid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UberDuckFileResponse {
    path: Option<String>,
    started_at: Option<String>,
    failed_at: Option<String>,
    finished_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    pub category: String,
    pub display_name: String,
    pub model_id: String,
    pub name: String,
}

pub fn twitch_chat_filename(username: String, voice: String) -> String {
    let now: DateTime<Utc> = Utc::now();

    format!("{}_{}_{}", now.timestamp(), username, voice)
}

#[async_trait]
impl EventHandler for UberDuckHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UberDuckRequest(msg) => msg,
                _ => continue,
            };

            let ch = msg.message.chars().next().unwrap();
            if ch == '!' {
                continue;
            };

            println!("We are trying for an Uberduck request: {}", msg.voice);

            let stream_character = stream_character::build_stream_character(
                &self.pool,
                &msg.username,
            )
            .await?;
            println!("\n\tStream Character: {:?}\n", stream_character);

            let source = match msg.source {
                Some(source) => source,
                None => stream_character.source.clone(),
            };

            let (username, secret) = uberduck_creds();

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.uberduck.ai/speak")
                .basic_auth(username.clone(), Some(secret.clone()))
                .json(&[
                    ("speech", msg.voice_text),
                    ("voice", msg.voice.clone()),
                ])
                .send()
                .await?
                .json::<UberDuckVoiceResponse>()
                .await?;

            let uuid = match res.uuid {
                Some(x) => x,
                None => continue,
            };

            loop {
                let url = format!(
                    "https://api.uberduck.ai/speak-status?uuid={}",
                    &uuid
                );

                let (username, secret) = uberduck_creds();
                let response = client
                    .get(url)
                    .basic_auth(username, Some(secret))
                    .send()
                    .await?;

                // TODO: abstract this out
                // Show Loading Duck
                let _ = tx.send(Event::SourceVisibilityRequest(
                    SourceVisibilityRequest {
                        scene: "Characters".to_string(),
                        source: "loading_duck".to_string(),
                        enabled: true,
                    },
                ));

                let text = response.text().await?;
                let file_resp: UberDuckFileResponse =
                    serde_json::from_str(&text)?;
                println!("Uberduck Finished at: {:?}", file_resp.finished_at);

                match file_resp.path {
                    Some(new_url) => {
                        let _ = tx.send(Event::SourceVisibilityRequest(
                            // Abstract these values out
                            SourceVisibilityRequest {
                                scene: "Characters".to_string(),
                                source: "loading_duck".to_string(),
                                enabled: false,
                            },
                        ));

                        let text_source = format!("{}-text", source.clone());
                        let _ = tx.send(Event::TransformOBSTextRequest(
                            TransformOBSTextRequest {
                                message: msg.message.clone(),
                                text_source,
                            },
                        ));

                        let filename =
                            twitch_chat_filename(msg.username, msg.voice);
                        let full_filename = format!("{}.wav", filename);

                        println!("Trying to Save: {}", full_filename);
                        let local_path = format!(
                            "./TwitchChatTTSRecordings/{}",
                            full_filename
                        );
                        let response = client.get(new_url).send().await?;
                        let file = File::create(local_path.clone())?;
                        let mut writer = BufWriter::new(file);
                        writer.write_all(&response.bytes().await?)?;
                        println!("Downloaded File From Uberduck, Playing Soon: {:?}!", local_path);

                        let _ = tx.send(Event::StreamCharacterRequest(
                            StreamCharacterRequest {
                                source: source.clone(),
                                enabled: true,
                            },
                        ));

                        let file =
                            BufReader::new(File::open(local_path).unwrap());
                        self.sink.append(
                            Decoder::new(BufReader::new(file)).unwrap(),
                        );
                        self.sink.sleep_until_end();

                        // THIS IS HIDING THE PERSON AFTER
                        // We might want to wait a little longer, then hide
                        // we could also kick off a hide event
                        let ten_millis = time::Duration::from_millis(1000);

                        thread::sleep(ten_millis);

                        let source = source.clone();
                        let _ = tx.send(Event::StreamCharacterRequest(
                            StreamCharacterRequest {
                                source,
                                enabled: false,
                            },
                        ));
                        break;
                    }
                    None => {
                        // Wait 1 second before seeing if the file is ready.
                        let ten_millis = time::Duration::from_millis(1000);
                        thread::sleep(ten_millis);
                    }
                }
            }
        }
    }
}

fn uberduck_creds() -> (String, String) {
    let username = env::var("UBER_DUCK_KEY")
        .expect("Failed to read UBER_DUCK_KEY environment variable");
    let secret = env::var("UBER_DUCK_SECRET")
        .expect("Failed to read UBER_DUCK_SECRET environment variable");
    (username, secret)
}

// ======================================

// All 6 Filters
// I think we should try alternative filter triggering instead
// we need to trigger 3 filters each time
// and we can get the names based off a pattern
// This is not the ideal method
pub fn find_obs_character(_voice: &str) -> &str {
    // TODO
    //      - first username
    //      - then voice
    //
    //
    let default_source = obs::DEFAULT_STREAM_CHARACTER_SOURCE;

    default_source
}

pub async fn set_voice(
    voice: String,
    username: String,
    pool: &sqlx::PgPool,
) -> Result<()> {
    let model = stream_character::user_stream_character_information::Model {
        username: username.clone(),
        voice: voice.to_string(),
        // This should be abstracted
        obs_character: "Seal".to_string(),
        random: false,
    };
    model.save(pool).await?;

    Ok(())
}

pub async fn talk_in_voice(
    contents: String,
    voice: String,
    username: String,
    tx: &broadcast::Sender<Event>,
) -> Result<()> {
    let spoken_string =
        contents.clone().replace(&format!("!voice {}", &voice), "");

    if spoken_string == "" {
        return Ok(());
    }

    let seal_text = match obs_text::breakup_text(contents.clone()).await {
        Ok(seal_text) => seal_text,
        Err(_) => return Ok(()),
    };

    let voice_text = spoken_string.clone();
    println!("We trying for the voice: {} - {}", voice, voice_text);
    let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
        voice: voice.to_string(),
        message: seal_text,
        voice_text,
        username,
        source: None,
    }));
    Ok(())
}

pub async fn use_random_voice(
    _contents: String,
    username: String,
    // tx: &broadcast::Sender<Event>,
) -> Result<()> {
    let contents = fs::read_to_string("data/voices.json").unwrap();
    let voices: Vec<Voice> = serde_json::from_str(&contents).unwrap();

    // THIS SUCKs
    // let mut rng = rand::task_rng();

    // If I add this it all fails
    // let mut rng = thread_rng();
    // let random_index = rng.gen_range(0..voices.len());
    // let random_voice = &voices[random_index];
    //
    // I can't add this line in an async function
    // let mut rng = rand::thread_rng();
    let random_voice = &voices[0];

    println!("Random Voice Chosen: {:?}", random_voice);

    let spoken_string = contents.clone().replace("!random", "");

    let seal_text = match obs_text::breakup_text(contents.clone()).await {
        Ok(seal_text) => seal_text,
        Err(_) => return Ok(()),
    };

    let voice_text = spoken_string.clone();

    // let _ = tx.send(Event::TransformOBSTextRequest(TransformOBSTextRequest {
    //     message: random_voice.name.clone(),
    //     text_source: "Soundboard-Text".to_string(),
    // }));

    // let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
    //     voice: random_voice.name.clone(),
    //     message: seal_text,
    //     voice_text,
    //     username,
    //     source: None,
    // }));
    Ok(())
}
