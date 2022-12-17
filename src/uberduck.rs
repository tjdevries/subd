use anyhow::Result;
use async_trait::async_trait;
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
use subd_types::TransformOBSTextRequest;
use subd_types::TriggerHotkeyRequest;
use tokio::sync::broadcast;

pub struct UberDuckHandler {
    pub sink: Sink,
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

// We could get rid of this
#[derive(Debug)]
pub struct CharacterSetup {
    on: &'static str,
    off: &'static str,
    source: &'static str,
    text_source: &'static str,
}

// Should they be optional???
pub struct StreamCharacter {
    source: String,
    text_source: String,
    hotkey_on: String,
    hotkey_off: String,
    voice: String,
    username: String,
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

            let stream_character = build_stream_character(&msg.username);

            let (username, secret) = uberduck_creds();

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.uberduck.ai/speak")
                .basic_auth(username.clone(), Some(secret.clone()))
                .json(&[
                    ("speech", msg.voice_text),
                    ("voice", stream_character.voice.clone()),
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

                let text = response.text().await?;
                println!("Uberduck Response: {:?}", text);
                let file_resp: UberDuckFileResponse =
                    serde_json::from_str(&text)?;

                match file_resp.path {
                    Some(new_url) => {
                        // TODO Should we change this file name
                        // Make it unique
                        let local_path = "./test.wav";
                        let response = client.get(new_url).send().await?;
                        let file = File::create(local_path)?;
                        let mut writer = BufWriter::new(file);
                        writer.write_all(&response.bytes().await?)?;
                        println!("Downloaded File From Uberduck, Playing Soon: {:?}!", local_path);

                        let _ = tx.send(Event::TransformOBSTextRequest(
                            TransformOBSTextRequest {
                                message: msg.message,
                                text_source: stream_character
                                    .text_source
                                    .to_string(),
                            },
                        ));
                        let _ = tx.send(Event::TriggerHotkeyRequest(
                            TriggerHotkeyRequest {
                                hotkey: stream_character.hotkey_on.to_string(),
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
                        let _ = tx.send(Event::TriggerHotkeyRequest(
                            TriggerHotkeyRequest {
                                hotkey: stream_character.hotkey_off.to_string(),
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

// Character Builder
// Then Just use that
fn build_stream_character(username: &str) -> StreamCharacter {
    // Start with username
    //
    // Username picks Voice
    //
    // Voice picks Source and Hotkeys

    let default_voice = "brock-samson";
    // let default_voice = "danny-devito-angry";
    // let default_voice = "goku";
    // let default_voice = "mickey-mouse";
    // let default_voice = "mojo-jojo";
    // let default_voice = "tommy-pickles";

    let voices2: HashMap<&str, &str> = HashMap::from([
        ("beginbotbot", "mr-krabs-joewhyte"),
        // ("beginbotbot", "theneedledrop"),
        // ("beginbot", "danny-devito-angry"),
        ("beginbot", "big-gay-al"),
        ("ArtMattDank", "dr-nick"),
        // ("ArtMattDank", "mojo-jojo"),
        ("carlvandergeest", "danny-devito-angry"),
        ("stupac62", "stewie-griffin"),
        ("swenson", "mike-wazowski"),
        ("teej_dv", "mr-krabs-joewhyte"),
        // ("theprimeagen", "big-gay-al"),
    ]);

    let voice = match voices2.get(username) {
        Some(v) => v,
        None => default_voice,
    };

    let character = find_obs_character(voice);

    StreamCharacter {
        username: username.to_string(),
        voice: voice.to_string(),
        text_source: character.text_source.to_string(),
        source: character.source.to_string(),
        hotkey_on: character.on.to_string(),
        hotkey_off: character.off.to_string(),
    }
}
// All 6 Filters

// This is not ideal though
// I think we should try alternative filter triggering instead
// we need to trigger 3 filters each time
// and we can get the names based offa  pattern
// This is not the ideal method
fn find_obs_character(voice: &str) -> CharacterSetup {
    // TODO: We need one of these for each voice
    let mut hotkeys: HashMap<&str, CharacterSetup> = HashMap::from([
        (
            "mr-krabs-joewhyte",
            CharacterSetup {
                on: "OBS_KEY_0",
                off: "OBS_KEY_1",
                text_source: "mr.crabs-text",
                source: "mr.crabs",
            },
        ),
        (
            "danny-devito-angry",
            CharacterSetup {
                on: "OBS_KEY_2",
                off: "OBS_KEY_3",
                source: "Kevin",
                text_source: "Kevin-text",
            },
        ),
    ]);

    let default_hotkeys = CharacterSetup {
        on: "OBS_KEY_6",
        off: "OBS_KEY_7",
        source: "Seal",
        text_source: "Text",
    };

    match hotkeys.remove(voice) {
        Some(v) => v,
        None => default_hotkeys,
    }
}
