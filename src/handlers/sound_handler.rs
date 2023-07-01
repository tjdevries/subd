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

// Define a custom data structure to hold the values
#[derive(Serialize)]
struct Record {
    field_1: String,
    field_2: String,
}

fn write_records_to_csv(path: &str, records: &[Record]) -> Result<()> {
    let mut writer = Writer::from_path(path)?;

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;

    Ok(())
}

// Looks through raw-text to either play TTS or play soundeffects
#[async_trait]
impl EventHandler for SoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let paths = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for path in paths {
            mp3s.insert(path.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;
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

            // if msg.roles.is_twitch_staff() {

            let spoken_string = msg.contents.clone();
            let voice_text = msg.contents.to_string();
            let _speech_bubble_text = uberduck::chop_text(spoken_string);

            // Anything less than 3 words we don't use
            let split = voice_text.split(" ");
            let vec = split.collect::<Vec<&str>>();
            if vec.len() < 2 {
                continue;
            };

            // What does this do?
            let stream_character =
                uberduck::build_stream_character(&self.pool, &msg.user_name)
                    .await?;

            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;

            let voice = stream_character.voice.clone();

            // voice: stream_character.voice,
            let mut character = Character {
                voice: Some(voice),
                ..Default::default()
            };

            // See if I'm none of these!!!!
            //
            // This is all about how to respond to messages from various
            // types of users
            if msg.roles.is_twitch_staff() {
                character.voice =
                    Some(obs::TWITCH_STAFF_OBS_SOURCE.to_string());
                character.source =
                    Some(obs::TWITCH_STAFF_VOICE.to_string());
            } else if msg.user_name == "beginbotbot" {
                // TODO: Get better voice
                character.voice =
                    Some(obs::TWITCH_HELPER_VOICE.to_string());
                // character.voice = Some("stephen-a-smith".to_string());
                // Some("stephen-a-smith".to_string())
            } else if msg.roles.is_twitch_mod() {
                // character.voice =
                //     Some(server::obs::TWITCH_MOD_DEFAULT_VOICE.to_string());
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
                    let records = vec![Record {
                        field_1: voice.clone(),
                        field_2: voice_text.clone(),
                    }];

                    // Write records to a CSV file
                    let csv_path =
                        "/home/begin/code/BeginGPT/tmp/voice_character.csv";
                    write_records_to_csv(&csv_path, &records)?;

                    // let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                    //     voice,
                    //     message: speech_bubble_text,
                    //     voice_text,
                    //     username: msg.user_name,
                    //     source: character.source,
                    // }));
                }
                None => {}
            }

            // If we have the implicit_soundeffects enabled
            // we go past this!
            if state.implicit_soundeffects {
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

                    // TODO: Look into using these!
                    // self.sink.volume()
                    // self.sink.set_volume()
                    // self.sink.len()

                    // We need this so we can allow to trigger the next word in OBS
                    // TODO: We should abstract
                    // and figure out a better way of determine the time
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
