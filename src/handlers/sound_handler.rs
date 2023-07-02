use anyhow::Result;
use csv::Writer;
use async_trait::async_trait;
use events::EventHandler;
use rodio::*;
use rodio::Decoder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::fs;
use std::io::BufReader;
use std::thread;
use std::time;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use crate::twitch_stream_state;
use crate::uberduck;
use crate::obs;
use tokio::sync::broadcast;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Character {
    pub voice: Option<String>,
    pub source: Option<String>,
}

pub struct SoundHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}

pub struct ExplicitSoundHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}


// Define a custom data structure to hold the values
#[derive(Serialize)]
struct Record {
    field_1: String,
    field_2: String,
}
#[async_trait]
impl EventHandler for ExplicitSoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // Get all soundeffects loaded up once
        // so we can search through them all
        let soundeffect_files = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for soundeffect_file in soundeffect_files {
            mp3s.insert(soundeffect_file.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;

            // This is meant to filter out messages
            // Right now it only filters Nightbot
            let msg = match event {
                Event::UserMessage(msg) => {
                    // TODO: Add a list here
                    if msg.user_name == "Nightbot" {
                        continue;
                    }
                    msg
                }
                _ => continue,
            };

            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;
            
            // Only continue if we have the implicit_soundeffects enabled
            if !state.explicit_soundeffects {
                continue;
            }

            let mut potential_sound = msg.contents.clone();
            let first_char = potential_sound.remove(0);
            if first_char != '!' {
                continue
            }
            let word = potential_sound;
 
            let text_source =
                obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string();
            
            let sanitized_word = word.to_lowercase();
            let full_name = format!("./MP3s/{}.mp3", sanitized_word);

            if mp3s.contains(&full_name) {
                let _ = tx.send(Event::TransformOBSTextRequest(
                    TransformOBSTextRequest {
                        message: sanitized_word.clone(),
                        text_source: text_source.to_string(),
                    },
                ));

                let file = BufReader::new(
                    File::open(format!("./MP3s/{}.mp3", sanitized_word))
                        .unwrap(),);
                self.sink
                    .append(Decoder::new(BufReader::new(file)).unwrap());

                self.sink.sleep_until_end();

                let sleep_time = time::Duration::from_millis(100);
                thread::sleep(sleep_time);
            }

            // This clears the OBS Text
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source: text_source.to_string(),
                },
            ));
        }
    }
    
}

// Looks through raw-text to either play TTS or play soundeffects
#[async_trait]
impl EventHandler for SoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {

        // Get all soundeffects loaded up once
        // so we can search through them all
        let soundeffect_files = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for soundeffect_file in soundeffect_files {
            mp3s.insert(soundeffect_file.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;

            // This is meant to filter out messages
            // Right now it only filters Nightbot
            let msg = match event {
                Event::UserMessage(msg) => {
                    // TODO: Add a list here
                    if msg.user_name == "Nightbot" {
                        continue;
                    }
                    msg
                }
                _ => continue,
            };

            let spoken_string = msg.contents.clone();
            let voice_text = msg.contents.to_string();
            let speech_bubble_text = uberduck::chop_text(spoken_string);

            // Anything less than 2 words we don't use
            let split = voice_text.split(" ");
            let vec = split.collect::<Vec<&str>>();
            if vec.len() < 2 {
                continue;
            };

            // This is how we determing the voice for the user
            let stream_character =
                uberduck::build_stream_character(&self.pool, &msg.user_name)
                    .await?;
            let voice = stream_character.voice.clone();
            println!("\nvoice from stream_character: {}", voice);
            

            // This is the current state of the stream:
            //    whether you are allowing all text to be read
            //    whether you are allowing soundeffects to happen automatically
            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;

            let mut character = Character {
                voice: Some(voice),
                ..Default::default()
            };

            println!("\n\tcharacter: {:?}", character);

            // This is all about how to respond to messages from various types of users
            if msg.roles.is_twitch_staff() {
                character.voice =
                    Some(obs::TWITCH_STAFF_OBS_SOURCE.to_string());
                character.source =
                    Some(obs::TWITCH_STAFF_VOICE.to_string());
            } else if msg.user_name == "beginbotbot" {
                character.voice =
                    Some(obs::TWITCH_HELPER_VOICE.to_string());
            } else if msg.roles.is_twitch_mod() {
                match character.voice {
                    Some(_) => { }
                    None => {
                        character.voice =
                            Some(obs::TWITCH_MOD_DEFAULT_VOICE.to_string());
                    }
                }
            } else if msg.roles.is_twitch_sub() {
                character.voice = Some(stream_character.voice.clone());
            } else if !state.sub_only_tts {
                // This is what everyone get's to speak with
                // if we are allowing non-subs to speak
                character.voice = Some(stream_character.voice.clone());
            }
            

            // If the character
            // If we have a voice assigned, then we fire off an UberDuck Request
            match character.voice {
                Some(voice) => {
                    
                    // Write records to a CSV file
                    // let records = vec![Record {
                    //     field_1: voice.clone(),
                    //     field_2: voice_text.clone(),
                    // }];
                    // TODO: What are you doing Begin!
                    // let csv_path =
                    //     "/home/begin/code/BeginGPT/tmp/voice_character.csv";
                    // write_records_to_csv(&csv_path, &records)?;
                    //
                    // At this point it's brock-sampson
                    println!("\n\tvoice: {}", voice);

                    // The voice here isn't be respected
                    let _ = tx.send(Event::UberDuckRequest(subd_types::UberDuckRequest {
                        voice,
                        message: speech_bubble_text,
                        voice_text,
                        username: msg.user_name,
                        source: character.source,
                    }));
                }
                None => {}
            }

            // Only continue if we have the implicit_soundeffects enabled
            if !state.implicit_soundeffects {
                continue;
            }

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let text_source =
                obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string();
            for word in splitmsg {
                let sanitized_word = word.as_str().to_lowercase();
                let full_name = format!("./MP3s/{}.mp3", sanitized_word);

                if mp3s.contains(&full_name) {
                    let _ = tx.send(Event::TransformOBSTextRequest(
                        TransformOBSTextRequest {
                            message: sanitized_word.clone(),
                            text_source: text_source.to_string(),
                        },
                    ));

                    let file = BufReader::new(
                        File::open(format!("./MP3s/{}.mp3", sanitized_word))
                            .unwrap(),);
                    self.sink
                        .append(Decoder::new(BufReader::new(file)).unwrap());

                    self.sink.sleep_until_end();

                    let sleep_time = time::Duration::from_millis(100);
                    thread::sleep(sleep_time);
                }
            }

            // This clears the OBS Text
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source: text_source.to_string(),
                },
            ));
        }
    }
}

#[allow(dead_code)]
fn write_records_to_csv(path: &str, records: &[Record]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
}
