use crate::stream_character;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use events::EventHandler;
use rodio::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::{thread, time};
use subd_types::Event;
use subd_types::SourceVisibilityRequest;
use subd_types::StreamCharacterRequest;
use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;

pub const DEFAULT_STREAM_CHARACTER_SOURCE: &str = "Seal";

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

// Should they be optional???
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamCharacter {
    // text_source: String,
    pub voice: String,
    pub source: String,
    pub username: String,
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

            // right here we crashed
            // !voice weed-goblin
            let ch = msg.message.chars().next().unwrap();
            if ch == '!' {
                continue;
            };

            println!("We are trying for an Uberduck request: {}", msg.voice);

            // We determine character
            // entirely based on username
            // HERE WE LOOK UP!!!

            let stream_character =
                build_stream_character(&self.pool, &msg.username).await?;
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

                // Show Loading Duck
                let _ = tx.send(Event::SourceVisibilityRequest(
                    SourceVisibilityRequest {
                        scene: "Characters".to_string(),
                        source: "loading_duck".to_string(),
                        enabled: true,
                    },
                ));

                let text = response.text().await?;
                // we need to this to be better
                let file_resp: UberDuckFileResponse =
                    serde_json::from_str(&text)?;
                println!("Uberduck Finished at: {:?}", file_resp.finished_at);

                match file_resp.path {
                    Some(new_url) => {
                        let _ = tx.send(Event::SourceVisibilityRequest(
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

                        // So the filename is fucking up
                        // it's not unique
                        let filename =
                            twitch_chat_filename(msg.username, msg.voice);
                        let full_filename = format!("{}.wav", filename);

                        // I WANT TO SAVE THIS FILE
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

                        // Hmm We shouldn't fail here then
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
fn find_obs_character(voice: &str) -> &str {
    // This makes no sense
    let default_hotkeys = DEFAULT_STREAM_CHARACTER_SOURCE;

    // We need defaults for the source
    // TODO: We need one of these for each voice
    let mut hotkeys: HashMap<&str, &str> = HashMap::from([
        ("brock-samson", "Seal"),
        ("alex-jones", "Seal"),
        ("lil-jon", "Seal"),
        ("theneedledrop", "Birb"),
        ("richard-ayoade", "Kevin"),
        ("spongebob", "Kevin"),
        ("arbys", "Kevin"),
        ("slj", "Teej"),
        ("rodney-dangerfield", "Teej"),
        // ("theneedledrop", "Kevin"),
        // ("theneedledrop", "Seal"),
        // ("theneedledrop", "ArtMatt"),
        // ("mojo-jojo", "Birb"),
        ("mojo-jojo", "Teej"),
        // ("mojo-jojo", "ArtMatt"),
        // ("mojo-jojo", "Kevin"),
        ("mr-krabs-joewhyte", "Crabs"),
        ("danny-devito-angry", "Kevin"),
        ("stewie-griffin", "ArtMatt"),
        ("ross-geller", "ArtMatt"),
        ("rossmann", "ArtMatt"),
        ("c-3po", "C3PO"),
        ("carl-sagan", "Seal"),
        ("dr-robotnik-movie", "Randall"),
    ]);

    match hotkeys.remove(voice) {
        Some(v) => v,
        None => default_hotkeys,
    }
}

// Character Builder
// Then Just use that
pub async fn build_stream_character(
    pool: &sqlx::PgPool,
    username: &str,
) -> Result<StreamCharacter> {
    let default_voice = "arbys";

    let voice =
        match stream_character::get_voice_from_username(pool, username).await {
            Ok(voice) => voice,
            Err(_) => {
                return Ok(StreamCharacter {
                    username: username.to_string(),
                    voice: default_voice.to_string(),
                    source: DEFAULT_STREAM_CHARACTER_SOURCE.to_string(),
                })
            }
        };

    let character = find_obs_character(&voice);

    Ok(StreamCharacter {
        username: username.to_string(),
        voice: voice.to_string(),
        source: character.to_string(),
    })
}
