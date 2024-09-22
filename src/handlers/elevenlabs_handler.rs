use anyhow::Result;
use async_trait::async_trait;
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::{thread, time};
use stream_character;
use subd_audio;
use subd_elevenlabs;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

// Should this have an OBS Client as well
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
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let twitch_client = Arc::new(Mutex::new(self.twitch_client));
        let clone_twitch_client = twitch_client.clone();
        let _locked_client = clone_twitch_client.lock().await;

        let obs_client = Arc::new(Mutex::new(self.obs_client));
        let obs_client_clone = obs_client.clone();
        let _locked_obs_client = obs_client_clone.lock().await;

        loop {
            // This feels dumb
            let default_global_voice = "troy".to_string();
            let event = rx.recv().await?;

            let msg = match event {
                Event::ElevenLabsRequest(msg) => msg,
                _ => continue,
            };

            let ch = match msg.message.chars().next() {
                Some(ch) => ch,
                None => {
                    continue;
                }
            };
            if ch == '!' || ch == '@' {
                continue;
            };

            let pool_clone = self.pool.clone();

            let twitch_state = async {
                twitch_stream_state::get_twitch_state(&pool_clone).await
            };

            let is_global_voice_enabled = match twitch_state.await {
                Ok(state) => state.global_voice,
                Err(err) => {
                    eprintln!("Error fetching twitch_stream_state: {:?}", err);
                    false
                }
            };

            let global_voice = stream_character::get_voice_from_username(
                &self.pool, "beginbot",
            )
            .await
            .unwrap_or(default_global_voice);

            let user_voice_opt = stream_character::get_voice_from_username(
                &self.pool,
                msg.username.clone().as_str(),
            )
            .await;

            // We sometimes pass a voice with the message, for various effects
            // And we are overwriting the global voice because of that
            // Seems kind wrong
            let final_voice = match msg.voice {
                Some(voice) => {
                    if is_global_voice_enabled {
                        println!("Using Global Voice");
                        global_voice.clone()
                    } else {
                        voice
                    }
                }
                None => {
                    if is_global_voice_enabled {
                        println!("Using Global Voice");
                        global_voice.clone()
                    } else {
                        match user_voice_opt {
                            Ok(user_voice) => user_voice,
                            Err(_) => global_voice.clone(),
                        }
                    }
                }
            };

            let filename = subd_elevenlabs::twitch_chat_filename(
                msg.username.clone(),
                final_voice.clone(),
            );

            let chat_message =
                subd_elevenlabs::sanitize_chat_message(msg.message.clone());

            // We keep track if we choose a random name for the user,
            // so we can inform them on screen
            let mut is_random = false;

            let voice_data =
                subd_elevenlabs::find_voice_id_by_name(&final_voice);
            let (voice_id, voice_name) = match voice_data {
                Some((id, name)) => (id, name),
                None => {
                    is_random = true;
                    subd_elevenlabs::find_random_voice()
                }
            };

            // The voice here in the TTS body is final
            let tts_body = TtsBody {
                model_id: None,
                text: chat_message,
                voice_settings: None,
            };
            println!("Calling TTS");
            let tts_result = self.elevenlabs.tts(&tts_body, voice_id);
            let bytes = match tts_result {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("ElevenLabs TTS Error: {:?}", e);

                    // How do I not crash
                    continue;
                }
            };
            println!("Done Calling TTS");

            // w/ Extension
            let full_filename = format!("{}.wav", filename);
            // TODO: Don't reference begin's computer
            let tts_folder = "/home/begin/code/subd/TwitchChatTTSRecordings";
            let mut local_audio_path =
                format!("{}/{}", tts_folder, full_filename);

            if let Err(e) = std::fs::write(local_audio_path.clone(), bytes) {
                eprintln!("Error writing tts file: {:?}", e);
                continue;
            }

            if msg.reverb {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| {
                    subd_elevenlabs::add_reverb(audio_path.clone())
                });
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            }

            if let Some(stretch) = msg.stretch {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| {
                    subd_elevenlabs::stretch_audio(audio_path, stretch)
                });
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            }

            if let Some(pitch) = msg.pitch {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| {
                    subd_elevenlabs::change_pitch(audio_path, pitch)
                });
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            };

            if final_voice == "evil_pokimane" {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| {
                    subd_elevenlabs::change_pitch(
                        audio_path,
                        "-200".to_string(),
                    )
                })
                .and_then(|audio_path| subd_elevenlabs::add_reverb(audio_path));
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            }

            if final_voice == "satan" {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| {
                    subd_elevenlabs::change_pitch(
                        audio_path,
                        "-350".to_string(),
                    )
                })
                .and_then(|audio_path| subd_elevenlabs::add_reverb(audio_path));
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            }

            // What is the difference
            if final_voice == "god" {
                let res = subd_elevenlabs::normalize_tts_file(
                    local_audio_path.clone(),
                )
                .and_then(|audio_path| subd_elevenlabs::add_reverb(audio_path));
                if let Ok(audio_path) = res {
                    local_audio_path = audio_path
                };
            }

            // =====================================================
            // WE just send a mesage to chat, with the mood
            // and it's optional

            // We are supressing a whole bunch of alsa message
            let backup = subd_utils::redirect_stderr()
                .expect("Failed to redirect stderr");

            let (_stream, stream_handle) =
                subd_audio::get_output_stream("pulse").expect("stream handle");

            let onscreen_msg = format!(
                "{} | g: {} | r: {} | {}",
                msg.username, is_global_voice_enabled, is_random, voice_name
            );

            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: onscreen_msg,
                    text_source:
                        subd_types::consts::get_soundboard_text_source_name(),
                },
            ));
            let sink = rodio::Sink::try_new(&stream_handle).unwrap();

            // sink.set_volume(1.3);
            sink.set_volume(0.5);
            match final_voice.as_str() {
                "melkey" => sink.set_volume(1.0),
                "beginbot" => sink.set_volume(1.0),
                "evil_pokimane" => sink.set_volume(1.0),
                "satan" => sink.set_volume(0.7),
                "god" => sink.set_volume(0.7),
                _ => {
                    sink.set_volume(0.5);
                }
            };
            let f = match File::open(local_audio_path) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error opening tts file: {:?}", e);
                    continue;
                }
            };
            let file = BufReader::new(f);
            sink.append(Decoder::new(BufReader::new(file)).unwrap());

            sink.sleep_until_end();

            subd_utils::restore_stderr(backup);

            // This playsthe text
            let ten_millis = time::Duration::from_millis(1000);
            thread::sleep(ten_millis);
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source:
                        subd_types::consts::get_soundboard_text_source_name(),
                },
            ));
            thread::sleep(ten_millis);
        }
    }
}
