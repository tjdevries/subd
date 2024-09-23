use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use obws::Client as OBSClient;
use rand::{seq::SliceRandom, thread_rng};
use std::{collections::HashMap, fs};
use subd_types::AiScenesRequest;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub mod models;

pub async fn run_ai_scene(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    elevenlabs: &Elevenlabs,
    ai_scene_req: &AiScenesRequest,
) -> Result<()> {
    // Determine the voice to use for the voice-over
    let final_voice = determine_voice_to_use(
        &ai_scene_req.username,
        ai_scene_req.voice.clone(),
        pool,
    )
    .await?;

    // Generate and save the TTS audio
    let local_audio_path = generate_and_save_tts_audio(
        &final_voice,
        &ai_scene_req.username,
        &ai_scene_req.message,
        elevenlabs,
        ai_scene_req,
    )?;

    // Decide which scene to trigger based on the voice
    if let Some(image_file_path) =
        build_face_scene_request(&final_voice).await?
    {
        println!("Triggering AI Friend Scene");
        ai_friends::trigger_ai_friend(
            obs_client,
            twitch_client,
            ai_scene_req,
            image_file_path,
            local_audio_path,
            final_voice,
        )
        .await?;
    } else {
        println!("Triggering AI Movie Trailer Scene");
        ai_movie_trailers::trigger_movie_trailer(
            ai_scene_req,
            twitch_client,
            local_audio_path,
        )
        .await?;
    }

    Ok(())
}

fn generate_and_save_tts_audio(
    voice: &str,
    username: &str,
    chat_message: &str,
    elevenlabs: &Elevenlabs,
    ai_scenes_request: &AiScenesRequest,
) -> Result<String> {
    // Find the voice ID based on the voice name
    let (voice_id, _) =
        find_voice_id_by_name(voice).unwrap_or_else(find_random_voice);

    // Create the TTS request body
    let tts_body = TtsBody {
        model_id: None,
        text: chat_message.to_string(),
        voice_settings: None,
    };

    // Call the ElevenLabs TTS API
    let bytes = elevenlabs
        .tts(&tts_body, voice_id)
        .map_err(|e| anyhow!("Error calling ElevenLabs: {}", e))?;

    // Generate the filename and save path
    let filename = twitch_chat_filename(username, voice);
    let full_filename = format!("{}.wav", filename);
    let tts_folder = "/home/begin/code/subd/TwitchChatTTSRecordings";
    let local_audio_path = format!("{}/{}", tts_folder, full_filename);

    // Save the audio file locally
    fs::write(&local_audio_path, bytes)?;

    // Apply any voice modifiers
    subd_audio::add_voice_modifiers(
        ai_scenes_request,
        voice.to_string(),
        local_audio_path,
    )
}

async fn determine_voice_to_use(
    username: &str,
    voice_override: Option<String>,
    pool: &sqlx::PgPool,
) -> Result<String> {
    // If a voice override is provided, use it
    if let Some(voice) = voice_override {
        return Ok(voice);
    }

    // Check if the global voice setting is enabled
    if let Ok(state) = twitch_stream_state::get_twitch_state(pool).await {
        if state.global_voice {
            let global_voice =
                stream_character::get_voice_from_username(pool, "beginbot")
                    .await?;
            return Ok(global_voice);
        }
    }

    // Get the voice associated with the username
    let user_voice =
        stream_character::get_voice_from_username(pool, username).await?;
    Ok(user_voice)
}

fn find_voice_id_by_name(name: &str) -> Option<(String, String)> {
    // Read the list of voices from a JSON file
    let data = fs::read_to_string("data/voices.json")
        .expect("Unable to read voices.json");
    let voice_list: models::VoiceList =
        serde_json::from_str(&data).expect("Failed to parse voices.json");

    // Search for the voice by name (case-insensitive)
    let name_lowercase = name.to_lowercase();
    voice_list
        .voices
        .into_iter()
        .find(|voice| voice.name.to_lowercase() == name_lowercase)
        .map(|voice| (voice.voice_id, voice.name))
}

fn find_random_voice() -> (String, String) {
    // Read the list of voices from a JSON file
    let data = fs::read_to_string("data/voices.json")
        .expect("Unable to read voices.json");
    let voice_list: models::VoiceList =
        serde_json::from_str(&data).expect("Failed to parse voices.json");

    // Select a random voice
    let mut rng = thread_rng();
    let random_voice = voice_list
        .voices
        .choose(&mut rng)
        .expect("List of voices is empty");

    (random_voice.voice_id.clone(), random_voice.name.clone())
}

fn twitch_chat_filename(username: &str, voice: &str) -> String {
    let now: DateTime<Utc> = Utc::now();
    format!("{}_{}_{}", now.timestamp(), username, voice)
}

async fn build_face_scene_request(voice: &str) -> Result<Option<String>> {
    let voice_to_face_image: HashMap<&str, &str> = HashMap::from([
        ("satan", "archive/satan.png"),
        ("god", "archive/god.png"),
        ("ethan", "archive/alex_jones.png"),
        ("prime", "archive/green_prime.png"),
        ("teej", "archive/teej_5.png"),
        ("melkey", "archive/melkey.png"),
    ]);

    Ok(voice_to_face_image.get(voice).map(|&path| path.to_string()))
}

fn _sanitize_chat_message(raw_msg: &str) -> String {
    raw_msg
        .split_whitespace()
        .map(|word| {
            if word.contains("http") {
                "U.R.L".to_string()
            } else if word.len() > 50 {
                "long word".to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
