use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use events::EventHandler;
use rodio::Decoder;
use rodio::*;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use subd_types::Event;
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
        // Load all soundeffects so we can search through
        // move to a function
        let soundeffect_files = fs::read_dir("./MP3s").unwrap();
        let mut mp3s: HashSet<String> = vec![].into_iter().collect();
        for soundeffect_file in soundeffect_files {
            mp3s.insert(soundeffect_file.unwrap().path().display().to_string());
        }

        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => {
                    if msg.user_name == "Nightbot" {
                        continue;
                    }
                    msg
                }
                _ => continue,
            };
            let state =
                twitch_stream_state::get_twitch_state(&self.pool).await?;

            // Only continue if we have the explicit enabled
            if !state.explicit_soundeffects {
                continue;
            }

            // ======================================
            // Now we can maybe parse
            // ======================================

            let mut potential_sound = msg.contents.clone().to_lowercase();
            let first_char = potential_sound.remove(0);
            if first_char != '!' {
                continue;
            }
            let full_name = format!("./MP3s/{}.mp3", potential_sound);
            if !mp3s.contains(&full_name) {
                continue;
            };

            let mp3 =
                match File::open(format!("./MP3s/{}.mp3", potential_sound)) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error opening file: {}", e);
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

            println!("{} {}", "Playing Sound: {}".cyan(), full_name);
            self.sink.set_volume(0.5);
            self.sink.append(sound);
            // We don't need to sleep until the end necessarily
            // But this also makes sure another sound isn't trigger at the same time
            // but that could be desirable
            // for instance for lots of !claps
            self.sink.sleep_until_end();
        }
    }
}
