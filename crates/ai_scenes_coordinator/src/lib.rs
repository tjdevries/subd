use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use ai_friends;
use ai_movie_trailers;
use anyhow::anyhow;
use anyhow::Result;
use chrono::{DateTime, Utc};
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
// use obws::Client as OBSClient;
use rand::{seq::SliceRandom, thread_rng};
use rodio::*;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use stream_character;
use subd_audio;
use subd_types::AiScenesRequest;
use tokio::sync::Mutex;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

use obws::Client as OBSClient;

// time to write some Rust!
pub async fn run_ai_scene(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    elevenlabs: &Elevenlabs,
    ai_scene_req: &AiScenesRequest,
) -> Result<()> {
    // Figure out the voice for the voice-over
    let final_voice = determine_voice_to_use(
        ai_scene_req.username.clone(),
        ai_scene_req.voice.clone(),
        pool.clone(),
    )
    .await?;

    // Generate the Audio to for the Voice Over
    let filename = twitch_chat_filename(
        ai_scene_req.username.clone(),
        final_voice.clone(),
    );
    let chat_message = sanitize_chat_message(ai_scene_req.message.clone());
    let local_audio_path = generate_and_save_tts_audio(
        final_voice.clone(),
        filename,
        chat_message,
        &elevenlabs,
        &ai_scene_req,
    )
    .map_err(|e| {
        // I could add more info to this error?
        // anyhow Context???
        println!("Failed to generate audio: {}", e);
        e
    })?;

    // Trigger the background music
    println!("AI Scene Request {:?}", &ai_scene_req);
    // we are going to turn this off for now
    // if let Some(prompt) = &ai_scene_req.prompt {
    //     twitch_stream_state::set_ai_background_theme(&pool, &prompt).await?;
    // };
    let twitch_client = Arc::new(Mutex::new(twitch_client));
    let clone_twitch_client = twitch_client.clone();
    let locked_twitch_client = clone_twitch_client.lock().await;
    // let obs_client = Arc::new(Mutex::new(obs_client));
    // let obs_client_clone = obs_client.clone();
    // let locked_obs_client = obs_client_clone.lock().await;

    // This is a dumb way to decide if the scene is AI Friend or not
    // this only works because we are correlating a voice like prime, with asking prime a question
    // through channel points
    let face_image = build_face_scene_request(final_voice.clone()).await?;
    match face_image {
        Some(image_file_path) => {
            println!("Triggering AI Friend Scene");
            ai_friends::trigger_ai_friend(
                obs_client,
                &locked_twitch_client,
                ai_scene_req,
                image_file_path,
                local_audio_path,
                final_voice,
            )
            .await?
        }
        None => {
            println!("Triggering AI Movie");
            ai_movie_trailers::trigger_movie_trailer(
                ai_scene_req,
                &locked_twitch_client,
                local_audio_path,
            )
            .await?
        }
    }

    return Ok(());
}

fn generate_and_save_tts_audio(
    voice: String,
    filename: String,
    chat_message: String,
    elevenlabs: &Elevenlabs,
    ai_scenes_request: &AiScenesRequest,
) -> Result<String> {
    let voice_data = find_voice_id_by_name(&voice.clone());
    let (voice_id, _voice_name) = match voice_data {
        Some((id, name)) => (id, name),
        None => find_random_voice(),
    };

    // The voice here in the TTS body is final
    let tts_body = TtsBody {
        model_id: None,
        text: chat_message,
        voice_settings: None,
    };
    let tts_result = elevenlabs.tts(&tts_body, voice_id);
    let bytes =
        tts_result.map_err(|e| anyhow!("Error calling ElevenLabs: {}", e))?;

    // w/ Extension
    let full_filename = format!("{}.wav", filename);

    // TODO: remove begin references
    let tts_folder = "/home/begin/code/subd/TwitchChatTTSRecordings";
    let local_audio_path = format!("{}/{}", tts_folder, full_filename);
    std::fs::write(local_audio_path.clone(), bytes)?;

    subd_audio::add_voice_modifiers(ai_scenes_request, voice, local_audio_path)
}

// ================= //
// Finding Functions //
// ================= //

// This is about voices
// TODO: FIX THIS ASAP
async fn determine_voice_to_use(
    username: String,
    voice_override: Option<String>,
    pool: sqlx::PgPool,
) -> Result<String> {
    let twitch_state = twitch_stream_state::get_twitch_state(&pool);
    let global_voice =
        stream_character::get_voice_from_username(&pool.clone(), "beginbot")
            .await?;

    match voice_override {
        Some(voice) => return Ok(voice),
        None => {
            if let Ok(state) = twitch_state.await {
                if state.global_voice {
                    return Ok(global_voice);
                }
            };

            let user_voice_opt = stream_character::get_voice_from_username(
                &pool.clone(),
                username.clone().as_str(),
            )
            .await;

            return Ok(match user_voice_opt {
                Ok(voice) => voice,
                Err(_) => global_voice.clone(),
            });
        }
    }
}

fn find_voice_id_by_name(name: &str) -> Option<(String, String)> {
    // We should replace this with an API call
    // or call it every once-in-a-while and "cache"
    let data =
        fs::read_to_string("data/voices.json").expect("Unable to read file");
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
    raw_msg
        .split_whitespace()
        .map(|word| {
            if word.contains("http") {
                "U.R.L".to_string()
            } else {
                word.to_string()
            }
        })
        .map(|word| {
            if word.len() > 50 {
                "long word".to_string()
            } else {
                word.to_string()
            }
        })
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

fn twitch_chat_filename(username: String, voice: String) -> String {
    let now: DateTime<Utc> = Utc::now();

    format!("{}_{}_{}", now.timestamp(), username, voice)
}

async fn build_face_scene_request(voice: String) -> Result<Option<String>> {
    let voice_to_face_image = HashMap::from([
        ("satan".to_string(), "archive/satan.png".to_string()),
        ("god".to_string(), "archive/god.png".to_string()),
        ("ethan".to_string(), "archive/alex_jones.png".to_string()),
        // We need a systen for multiple photos
        // ("teej".to_string(), "archive/teej.png".to_string()),
        // ("teej".to_string(), "archive/teej_2.jpg".to_string()),
        ("prime".to_string(), "archive/green_prime.png".to_string()),
        ("teej".to_string(), "archive/teej_3.png".to_string()),
        ("melkey".to_string(), "archive/melkey.png".to_string()),
    ]);
    Ok(voice_to_face_image.get(&voice).cloned())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScenes {
    pub scenes: Vec<AIScene>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScene {
    pub reward_title: String,
    pub base_prompt: String,
    pub base_dalle_prompt: String,
    pub voice: String,
    pub music_bg: String,
    pub cost: usize,
    pub id: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
pub struct ElevenlabsVoice {
    pub voice_id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct VoiceList {
    pub voices: Vec<ElevenlabsVoice>,
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
