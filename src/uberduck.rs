use crate::audio;
use std::process::Command;
use crate::obs;
use crate::stream_character;
use crate::redirect;
use crate::twitch_stream_state;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use events::EventHandler;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use rodio::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::{thread, time};
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
use subd_types::ElevenLabsRequest;
use tokio::sync::broadcast;


#[derive(Deserialize, Debug)]
struct ElevenlabsVoice {
    voice_id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct VoiceList {
    voices: Vec<ElevenlabsVoice>,
}

pub struct ElevenLabsHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
    pub elevenlabs: Elevenlabs,
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
impl EventHandler for ElevenLabsHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                // TODO: rename UberDuckRequest to ElevenLabsRequest
                Event::ElevenLabsRequest(msg) => msg,
                _ => continue,
            };

            let ch = msg.message.chars().next().unwrap();
            if ch == '!' {
                continue;
            };

            let pool_clone = self.pool.clone();

            let twitch_state = async {
                twitch_stream_state::get_twitch_state(&pool_clone).await
            };

            let is_global_voice_enabled = match twitch_state.await {
                Ok(state) => {
                    state.global_voice
                },
                Err(err) => {
                    eprintln!("Error fetching twitch_stream_state: {:?}", err);
                    false
                }
            };
            
            let voice = stream_character::get_voice_from_username(&self.pool, "beginbot").await?;
            let voice_data = find_voice_id_by_name(&voice);
            let (_global_voice_id, global_voice) = match voice_data {
                Some((id, name)) => {
                    (id, name)
                },
                None => {
                    ("".to_string(), "".to_string())
                },
            };

            let final_voice = if is_global_voice_enabled  {
                println!("Global Voice is enabled: {}", global_voice);
                global_voice
            } else {
                msg.voice.clone()
            };

            // /wo extension
            let filename =
                twitch_chat_filename(msg.username.clone(), final_voice.clone());

            // w/ Extension
            let full_filename = format!("{}.wav", filename);
            let mut local_audio_path = format!("/home/begin/code/subd/TwitchChatTTSRecordings/{}", full_filename);

            // We need to sanitize the message of links
            // text: msg.message.clone(),
            let chat_message = sanitize_chat_message(msg.message.clone());

            let tts_body = TtsBody {
                model_id: None,
                text: chat_message,
                voice_settings: None,
            };

            let mut is_random = false;

            let voice_data = find_voice_id_by_name(&final_voice);
            let (voice_id, voice_name) = match voice_data {
                Some((id, name)) => {
                    (id, name)
                },
                None => {
                    is_random = true;
                    find_random_voice()
                },
            };
            let tts_result = self.elevenlabs.tts(&tts_body, voice_id);
            let bytes = tts_result.unwrap();

            std::fs::write(local_audio_path.clone(), bytes).unwrap();
            
            if final_voice == "satan" {
                let reverb_path = add_reverb(filename.clone(), full_filename.clone(), local_audio_path).unwrap();
                let pitch_path = format!("/home/begin/code/subd/TwitchChatTTSRecordings/Reverb/{}_reverb_pitch.wav", filename.clone());
                change_pitch(reverb_path, pitch_path.clone(), "-350".to_string()).unwrap();
                local_audio_path = pitch_path
            }
            
            // What is the difference
            if final_voice == "god" {
                local_audio_path = add_reverb(filename, full_filename, local_audio_path).unwrap();
            }

            // =====================================================

            // We are supressing a whole bunch of alsa message
            let backup = redirect::redirect_stderr().expect("Failed to redirect stderr");
            
            let (_stream, stream_handle) =
                audio::get_output_stream("pulse").expect("stream handle");
            
            let onscreen_msg = format!("{} | g: {} | r: {} | {}", msg.username, is_global_voice_enabled, is_random, voice_name);
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: onscreen_msg,
                    text_source: obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string(),
                },
            ));
            let sink = rodio::Sink::try_new(&stream_handle).unwrap();

            // sink.set_volume(1.0);
            sink.set_volume(0.7);
            let file = BufReader::new(File::open(local_audio_path).unwrap());
            
            sink.append(Decoder::new(BufReader::new(file)).unwrap());
            sink.sleep_until_end();
            
            redirect::restore_stderr(backup);
            
            // This playsthe text
            let ten_millis = time::Duration::from_millis(1000);
            thread::sleep(ten_millis);
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "".to_string(),
                    text_source: obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string(),
                },
            ));
            thread::sleep(ten_millis);
        }
    }
}

pub fn chop_text(starting_text: String) -> String {
    let mut seal_text = starting_text.clone();

    let spaces: Vec<_> = starting_text.match_indices(" ").collect();
    let line_length_modifier = 20;
    let mut line_length_limit = 20;
    for val in spaces.iter() {
        if val.0 > line_length_limit {
            seal_text.replace_range(val.0..=val.0, "\n");
            line_length_limit = line_length_limit + line_length_modifier;
        }
    }

    seal_text
}

fn find_obs_character(_voice: &str) -> &str {
    let default_character = obs::DEFAULT_STREAM_CHARACTER_SOURCE;
    return default_character;
}

pub async fn set_voice(
    voice: String,
    username: String,
    pool: &sqlx::PgPool,
) -> Result<()> {
    let model = stream_character::user_stream_character_information::Model {
        username: username.clone(),
        voice: voice.to_string().to_lowercase(),
        obs_character: obs::DEFAULT_STREAM_CHARACTER_SOURCE.to_string(),
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

    let seal_text = chop_text(spoken_string.clone());

    let voice_text = spoken_string.clone();
    println!("We trying for the voice: {} - {}", voice, voice_text);
    let _ = tx.send(Event::ElevenLabsRequest(ElevenLabsRequest {
        voice: voice.to_string(),
        message: seal_text,
        voice_text,
        username,
        source: None,
    }));
    Ok(())
}

pub async fn use_random_voice(
    contents: String,
    username: String,
    tx: &broadcast::Sender<Event>,
) -> Result<()> {
    let voices_contents = fs::read_to_string("data/voices.json").unwrap();
    let voices: Vec<Voice> = serde_json::from_str(&voices_contents).unwrap();
    let mut rng = thread_rng();
    let random_index = rng.gen_range(0..voices.len());
    let random_voice = &voices[random_index];

    let spoken_string = contents.clone().replace("!random", "");
    let speech_bubble_text = chop_text(spoken_string.clone());
    let voice_text = spoken_string.clone();

    let _ = tx.send(Event::TransformOBSTextRequest(TransformOBSTextRequest {
        message: random_voice.name.clone(),

        // TODO: This should probably be a different Text Source
        text_source: "Soundboard-Text".to_string(),
    }));

    let _ = tx.send(Event::ElevenLabsRequest(ElevenLabsRequest {
        voice: random_voice.name.clone(),
        message: speech_bubble_text,
        voice_text,
        username,
        source: None,
    }));
    Ok(())
}

pub async fn build_stream_character(
    pool: &sqlx::PgPool,
    username: &str,
) -> Result<StreamCharacter> {
    let default_voice = obs::TWITCH_DEFAULT_VOICE.to_string();

    let voice =
        match stream_character::get_voice_from_username(pool, username).await {
            Ok(voice) => voice,
            Err(_) => {
                println!("No Voice Found, Using Default");

                return Ok(StreamCharacter {
                    username: username.to_string(),
                    voice: default_voice.to_string(),
                    source: obs::DEFAULT_STREAM_CHARACTER_SOURCE.to_string(),
                });
            }
        };

    let character = find_obs_character(&voice);
    println!("Voice: {:?}", voice);

    Ok(StreamCharacter {
        username: username.to_string(),
        voice: voice.to_string(),
        source: character.to_string(),
    })
}

// ============= //
// Audio Effects //
// ============= //

fn change_pitch(reverb_path: String, pitch_path: String, pitch: String) -> Result<String> {
    Command::new("sox")
        .args(&["-t", "wav", &reverb_path, &pitch_path, "pitch", &pitch])
        .status()
        .expect("Failed to execute sox");
    Ok(pitch_path)
}

fn add_reverb(filename: String, full_filename: String, mut local_audio_path: String) -> Result<String> {
    let pre_reverb_file = format!("/home/begin/code/subd/TwitchChatTTSRecordings/Reverb/{}", full_filename);
    let ffmpeg_status = Command::new("ffmpeg")
        .args(&["-i", &local_audio_path, &pre_reverb_file])
        .status()
        .expect("Failed to execute ffmpeg");

    let final_output_path = format!("/home/begin/code/subd/TwitchChatTTSRecordings/Reverb/{}_reverb.wav", filename);
    if ffmpeg_status.success() {
        Command::new("sox")
            .args(&["-t", "wav", &pre_reverb_file, &final_output_path, "gain", "-2", "reverb", "70", "100", "50", "100", "10", "2"])
            .status()
            .expect("Failed to execute sox");
    }
    local_audio_path = final_output_path;
    Ok(local_audio_path)
}

// ================= //
// Finding Functions //
// ================= //

fn find_voice_id_by_name(name: &str) -> Option<(String, String)> {
    // Read JSON file (replace 'path_to_file.json' with your file's path)
    let data = fs::read_to_string("voices.json").expect("Unable to read file");

    // Deserialize JSON to VoiceList
    let voice_list: VoiceList =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    // Convert the input name to lowercase
    let name_lowercase = name.to_lowercase();

    // Iterate through voices and find the matching voice_id
    for voice in voice_list.voices {
        if voice.name.to_lowercase() == name_lowercase {
            return Some((voice.voice_id, voice.name));
        }
    }
    None
}

fn sanitize_chat_message(raw_msg: String) -> String {
    raw_msg.split_whitespace()
        .map(|word| if word.contains("http") {"U.R.L".to_string()} else { word.to_string() })
        .collect::<Vec<String>>()
        .join(" ")
}

fn find_random_voice() -> (String, String) {
    let data = fs::read_to_string("voices.json").expect("Unable to read file");

    let voice_list: VoiceList =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    let mut rng = thread_rng();
    let random_voice = voice_list
        .voices
        .choose(&mut rng)
        .expect("List of voices is empty");

    // Return both the voice ID and name
    (random_voice.voice_id.clone(), random_voice.name.clone())
}
