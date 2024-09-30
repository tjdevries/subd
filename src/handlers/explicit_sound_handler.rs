use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
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

pub struct ExplicitSoundHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl EventHandler for ExplicitSoundHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
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
                continue;
            }
            let word = potential_sound;
            let sanitized_word = word.to_lowercase();
            let full_name = format!("./MP3s/{}.mp3", sanitized_word);
            if !mp3s.contains(&full_name) {
                continue;
            };

            println!(
                "{} {}",
                "We found an sound to play! {}".cyan(),
                full_name
            );

            let mp3 = match File::open(format!("./MP3s/{}.mp3", sanitized_word))
            {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "Error opening sound file in ExplicitSoundHandler: {}",
                        e
                    );
                    continue;
                }
            };

            let file = BufReader::new(mp3.try_clone()?);
            let sound = match Decoder::new(BufReader::new(file)) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error decoding sound file: {}", e);
                    continue;
                }
            };

            self.sink.set_volume(0.5);
            self.sink.append(sound);
            // We don't need to sleep until the end necessarily
            self.sink.sleep_until_end();
        }
    }
}
