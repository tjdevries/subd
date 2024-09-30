use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use std::{fs::File, io::BufReader};
use stream_character;
use subd_elevenlabs;
use subd_types::{ElevenLabsRequest, Event};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

pub struct ElevenLabsHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pub elevenlabs: Elevenlabs,
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for ElevenLabsHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        println!("{}", "Kicking off ElevenLabsHandler".yellow());

        loop {
            let event = rx.recv().await?;

            if let Event::ElevenLabsRequest(msg) = event {
                self.process_elevenlabs_request(msg).await?;
            }
        }
    }
}

impl ElevenLabsHandler {
    async fn process_elevenlabs_request(
        &self,
        msg: ElevenLabsRequest,
    ) -> Result<()> {
        if self.should_skip_message(&msg.message) {
            return Ok(());
        }

        let voice = self.determine_final_voice(&msg).await?;
        let chat_message =
            subd_elevenlabs::sanitize_chat_message(msg.message.clone());
        let audio_path =
            self.process_audio(&voice, &chat_message, &msg).await?;

        self.play_audio(&audio_path, &voice);
        Ok(())
    }

    fn should_skip_message(&self, message: &str) -> bool {
        matches!(message.chars().next(), Some('!') | Some('@'))
    }

    async fn apply_audio_effects(
        &self,
        audio_path: String,
        final_voice: &str,
        msg: &ElevenLabsRequest,
    ) -> Result<String> {
        // Normalize the audio once
        let mut local_audio_path =
            match subd_elevenlabs::normalize_tts_file(audio_path.clone()) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Error normalizing tts file: {:?}", err);
                    // If normalization fails, we can decide to continue with the original path or return an error
                    return Ok(audio_path); // Proceeding with the original audio
                }
            };

        // Apply user-specified effects
        if msg.reverb {
            local_audio_path =
                subd_elevenlabs::add_reverb(local_audio_path.clone())
                    .unwrap_or_else(|err| {
                        eprintln!("Error adding reverb: {:?}", err);
                        local_audio_path.clone()
                    });
        }

        if let Some(stretch) = &msg.stretch {
            local_audio_path = subd_elevenlabs::stretch_audio(
                local_audio_path.clone(),
                stretch.clone(),
            )
            .unwrap_or_else(|err| {
                eprintln!("Error stretching audio: {:?}", err);
                local_audio_path.clone()
            });
        }

        if let Some(pitch) = &msg.pitch {
            local_audio_path = subd_elevenlabs::change_pitch(
                local_audio_path.clone(),
                pitch.clone(),
            )
            .unwrap_or_else(|err| {
                eprintln!("Error changing pitch: {:?}", err);
                local_audio_path.clone()
            });
        }

        // Apply special voice effects
        match final_voice {
            "evil_pokimane" => {
                local_audio_path = subd_elevenlabs::change_pitch(
                    local_audio_path.clone(),
                    "-200".to_string(),
                )
                .and_then(subd_elevenlabs::add_reverb)
                .unwrap_or_else(|err| {
                    eprintln!(
                        "Error processing 'evil_pokimane' voice: {:?}",
                        err
                    );
                    local_audio_path.clone()
                });
            }
            "satan" => {
                local_audio_path = subd_elevenlabs::change_pitch(
                    local_audio_path.clone(),
                    "-350".to_string(),
                )
                .and_then(subd_elevenlabs::add_reverb)
                .unwrap_or_else(|err| {
                    eprintln!("Error processing 'satan' voice: {:?}", err);
                    local_audio_path.clone()
                });
            }
            "god" => {
                local_audio_path =
                    subd_elevenlabs::add_reverb(local_audio_path.clone())
                        .unwrap_or_else(|err| {
                            eprintln!(
                                "Error processing 'god' voice: {:?}",
                                err
                            );
                            local_audio_path.clone()
                        });
            }
            _ => {}
        }

        Ok(local_audio_path)
    }

    async fn determine_final_voice(
        &self,
        msg: &ElevenLabsRequest,
    ) -> Result<String> {
        let default_global_voice = "ethan".to_string();

        let is_global_voice_enabled =
            match twitch_stream_state::get_twitch_state(&self.pool).await {
                Ok(state) => state.global_voice,
                Err(err) => {
                    eprintln!("Error fetching twitch_stream_state: {:?}", err);
                    false
                }
            };

        let global_voice =
            stream_character::get_voice_from_username(&self.pool, "beginbot")
                .await
                .unwrap_or_else(|_| default_global_voice.clone());

        let user_voice = stream_character::get_voice_from_username(
            &self.pool,
            &msg.username,
        )
        .await
        .unwrap_or_else(|_| global_voice.clone());

        let final_voice = if is_global_voice_enabled {
            global_voice.clone()
        } else {
            msg.voice.as_ref().unwrap_or(&user_voice).clone()
        };
        Ok(final_voice)
    }

    fn play_audio(&self, audio_path: &str, final_voice: &str) {
        self.sink.set_volume(match final_voice {
            "melkey" | "beginbot" | "evil_pokimane" => 1.0,
            "satan" | "god" => 0.7,
            _ => 0.5,
        });

        let file = match File::open(audio_path) {
            Ok(f) => BufReader::new(f),
            Err(e) => {
                eprintln!("Error opening tts file: {:?}", e);
                return;
            }
        };

        if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
            self.sink.append(decoder);
            self.sink.sleep_until_end();
        } else {
            eprintln!("Error decoding audio file");
        }
    }

    async fn process_audio(
        &self,
        final_voice: &str,
        chat_message: &str,
        msg: &ElevenLabsRequest,
    ) -> Result<String> {
        let filename = subd_elevenlabs::twitch_chat_filename(
            msg.username.clone(),
            final_voice.to_string(),
        );
        let voice_data = subd_elevenlabs::find_voice_id_by_name(final_voice)
            .unwrap_or_else(subd_elevenlabs::find_random_voice);
        let (voice_id, _voice_name) = voice_data;

        let tts_body = TtsBody {
            model_id: None,
            text: chat_message.to_string(),
            voice_settings: None,
        };

        let bytes = self.elevenlabs.tts(&tts_body, voice_id).map_err(|e| {
            eprintln!("ElevenLabs TTS Error: {:?}", e);
            anyhow!(e)
        })?;
        println!("Done Calling TTS");

        let full_filename = format!("{}.wav", filename);
        let tts_folder = "./TwitchChatTTSRecordings";
        let mut local_audio_path = format!("{}/{}", tts_folder, full_filename);

        std::fs::write(&local_audio_path, bytes).map_err(|e| {
            eprintln!("Error writing tts file: {:?}", e);
            e
        })?;

        local_audio_path = self
            .apply_audio_effects(local_audio_path.clone(), final_voice, msg)
            .await?;

        Ok(local_audio_path)
    }
}
