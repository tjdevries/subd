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
    pub voice: Option<String>,
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

            let ch = match msg.message.chars().next() {
                Some(ch) =>  ch,
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
                Ok(state) => {
                    state.global_voice
                },
                Err(err) => {
                    eprintln!("Error fetching twitch_stream_state: {:?}", err);
                    false
                }
            };
            let global_voice = stream_character::get_voice_from_username(&self.pool, "beginbot").await.unwrap();
            
            // Do we want explicit voice's passed to override global?
            // That means all !reverb !pitch etc. commands
            // If we passed in an explict voice use that
            let voice_name = match msg.voice {
                Some(voice) => voice,
                None =>{
                    global_voice.clone()
                }
            };
            
            let final_voice = if is_global_voice_enabled  {
                global_voice
            } else {
                voice_name
            };

            let filename =
                twitch_chat_filename(msg.username.clone(), final_voice.clone());

            let chat_message = sanitize_chat_message(msg.message.clone());

            // We keep track if we choose a random name for the user,
            // so we can inform them on screen
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
            
            // The voice here in the TTS body is final
            let tts_body = TtsBody {
                model_id: None,
                text: chat_message,
                voice_settings: None,
            };
            let tts_result = self.elevenlabs.tts(&tts_body, voice_id);
            let bytes = tts_result.unwrap();

            // w/ Extension
            let full_filename = format!("{}.wav", filename);
            let tts_folder = "/home/begin/code/subd/TwitchChatTTSRecordings";
            let mut local_audio_path = format!("{}/{}", tts_folder, full_filename);

            std::fs::write(local_audio_path.clone(), bytes).unwrap();
            
            if msg.reverb {
                local_audio_path = normalize_tts_file(local_audio_path.clone()).unwrap();
                local_audio_path = add_reverb(local_audio_path.clone()).unwrap();
            }

            match msg.stretch {
                Some(stretch) => {
                    local_audio_path = normalize_tts_file(local_audio_path.clone()).unwrap();
                    local_audio_path = stretch_audio(local_audio_path, stretch).unwrap();
                },
                None => {}
            }
            
            match msg.pitch {
                Some(pitch) => {
                    local_audio_path = normalize_tts_file(local_audio_path.clone()).unwrap();
                    local_audio_path = change_pitch(local_audio_path, pitch).unwrap();
                },
                None => {}
            }
            
            if final_voice == "satan" {
                local_audio_path = normalize_tts_file(local_audio_path.clone()).unwrap();
                local_audio_path = change_pitch(local_audio_path, "-350".to_string()).unwrap();
                local_audio_path = add_reverb(local_audio_path.clone()).unwrap();
            }
            
            // What is the difference
            if final_voice == "god" {
                local_audio_path = normalize_tts_file(local_audio_path.clone()).unwrap();
                local_audio_path = add_reverb(local_audio_path).unwrap();
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

            // sink.set_volume(1.3);
            sink.set_volume(0.5);
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
        voice: Some(voice.to_string()),
        message: seal_text,
        voice_text,
        username,
        ..Default::default()
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
        voice: Some(random_voice.name.clone()),
        message: speech_bubble_text,
        voice_text,
        username,
        ..Default::default()
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
                    voice: Some(default_voice.to_string()),
                    source: obs::DEFAULT_STREAM_CHARACTER_SOURCE.to_string(),
                });
            }
        };

    let character = find_obs_character(&voice);
    println!("Voice: {:?}", voice);

    Ok(StreamCharacter {
        username: username.to_string(),
        voice: Some(voice.to_string()),
        source: character.to_string(),
    })
}

// ============= //
// Audio Effects //
// ============= //

fn add_postfix_to_filepath(filepath: String, postfix: String) -> String {
    match filepath.rfind('.') {
        Some(index) => {
            let path = filepath[..index].to_string();
            let filename = filepath[index..].to_string();
            format!("{}{}{}", path, postfix, filename)
        }
        None => filepath,
    }
}


fn normalize_tts_file(local_audio_path: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path.clone(), "_norm".to_string());
    let ffmpeg_status = Command::new("ffmpeg")
        .args(&["-i", &local_audio_path, &audio_dest_path])
        .status()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_status.success() {
        Ok(audio_dest_path)
    } else {
        println!("Failed to normalize audio");
        Ok(local_audio_path)
    }
}


fn stretch_audio(local_audio_path: String, stretch: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path.clone(), "_stretch".to_string());
    Command::new("sox")
        .args(&["-t", "wav", &local_audio_path, &audio_dest_path, "stretch", &stretch])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

fn change_pitch(local_audio_path: String, pitch: String) -> Result<String> {
    let postfix = format!("{}_{}", "_pitch".to_string(), pitch);
    let audio_dest_path = add_postfix_to_filepath(local_audio_path.clone(), postfix);
    Command::new("sox")
        .args(&["-t", "wav", &local_audio_path, &audio_dest_path, "pitch", &pitch])
        .status()
        .expect("Failed to execute sox");
    
    Ok(audio_dest_path)
}


fn add_reverb(local_audio_path: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path.clone(), "_reverb".to_string());
    Command::new("sox")
        .args(&["-t", "wav", &local_audio_path, &audio_dest_path, "gain", "-2", "reverb", "70", "100", "50", "100", "10", "2"])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

// ================= //
// Finding Functions //
// ================= //

fn find_voice_id_by_name(name: &str) -> Option<(String, String)> {
    // We should replace this with an API call
    // or call it every once-in-a-while and "cache"
    let data = fs::read_to_string("voices.json").expect("Unable to read file");
    let voice_list: VoiceList =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    let name_lowercase = name.to_lowercase();

    for voice in voice_list.voices {
        if voice.name.to_lowercase() == name_lowercase {
            return Some((voice.voice_id, voice.name));
        }
    }
    None
}

fn sanitize_chat_message(raw_msg: String) -> String {
    // Let's replace any word longer than 50 characters
    raw_msg.split_whitespace()
        .map(|word| if word.contains("http") {"U.R.L".to_string()} else { word.to_string() })
        .map(|word| if word.len() > 50 { "long word".to_string() } else { word.to_string() })
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
