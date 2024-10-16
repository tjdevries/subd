use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use rodio::Decoder;
use rodio::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time;
use subd_elevenlabs;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;
use twitch_stream_state;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Character {
    pub voice: Option<String>,
    pub source: Option<String>,
}

pub struct ImplicitSoundHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}

// Looks through raw-text to either play TTS or play soundeffects
#[async_trait]
impl EventHandler for ImplicitSoundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // Get all soundeffects loaded up once
        // so we can search through them all
        let soundeffect_files = fs::read_dir("./MP3s").map_err(|e| {
            anyhow::anyhow!("Failed to read MP3s directory: {}", e)
        })?;
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for soundeffect_file in soundeffect_files {
            if let Ok(file) = soundeffect_file {
                mp3s.insert(file.path().display().to_string());
            }
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

            let spoken_string = &msg.contents;
            let speech_bubble_text = subd_elevenlabs::chop_text(spoken_string);

            // Anything less than 2 words we don't use
            let split = spoken_string.split(' ');
            let vec = split.collect::<Vec<&str>>();
            if vec.len() < 2 {
                continue;
            };

            // This is how we determing the voice for the user
            let stream_character = subd_elevenlabs::build_stream_character(
                &self.pool,
                &msg.user_name,
            )
            .await?;

            let voice = match stream_character.voice {
                Some(voice) => voice,
                None => subd_types::consts::get_twitch_mod_default_voice(),
            };

            // This is the current state of the stream:
            //    whether you are allowing all text to be read
            //    whether you are allowing soundeffects to happen automatically
            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;

            let mut character = Character {
                voice: Some(voice.clone()),
                ..Default::default()
            };

            // This is all about how to respond to messages from various types of users
            if msg.roles.is_twitch_staff() {
                character.voice =
                    Some(subd_types::consts::get_twitch_staff_voice());
                character.source =
                    Some(subd_types::consts::get_twitch_broadcaster_raw());
            } else if msg.user_name == "beginbotbot" {
                character.voice =
                    Some(subd_types::consts::get_twitch_helper_voice());
            } else if msg.roles.is_twitch_mod() {
                match character.voice {
                    Some(_) => {}
                    None => {
                        character.voice = Some(
                            subd_types::consts::get_twitch_mod_default_voice(),
                        );
                    }
                }
            } else if msg.roles.is_twitch_sub() || !state.sub_only_tts {
                character.voice = Some(voice);
            }

            //-            let spoken_string = msg.contents.clone();
            let voice_text = msg.contents.to_string();

            // If the character
            // If we have a voice assigned, then we fire off an elevenlabs Request
            if let Some(voice) = character.voice {
                let _ = tx.send(Event::ElevenLabsRequest(
                    subd_types::ElevenLabsRequest {
                        voice: Some(voice),
                        message: speech_bubble_text,
                        voice_text,
                        username: msg.user_name,
                        source: character.source,
                        ..Default::default()
                    },
                ));
            }

            // Only continue if we have the implicit_soundeffects enabled
            if !state.implicit_soundeffects {
                continue;
            }

            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let text_source =
                subd_types::consts::get_soundboard_text_source_name();

            // This is looking to play SFX
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

                    let file = match File::open(format!(
                        "./MP3s/{}.mp3",
                        sanitized_word
                    )) {
                        Ok(file) => BufReader::new(file),
                        Err(e) => {
                            println!("Error opening file: {}", e);
                            continue;
                        }
                    };

                    let (_stream, stream_handle) =
                        subd_audio::get_output_stream("pulse")
                            .expect("stream handle");

                    // TODO: We shouldn't be creating more and more handlers
                    let sink = match rodio::Sink::try_new(&stream_handle) {
                        Ok(sink) => sink,
                        Err(e) => {
                            println!("Error creating sink: {}", e);
                            continue;
                        }
                    };

                    match Decoder::new(BufReader::new(file)) {
                        Ok(source) => sink.append(source),
                        Err(e) => {
                            println!("Error decoding file: {}", e);
                            continue;
                        }
                    };

                    sink.sleep_until_end();

                    let sleep_time = time::Duration::from_millis(100);
                    thread::sleep(sleep_time);
                }
            }
        }
    }
}
