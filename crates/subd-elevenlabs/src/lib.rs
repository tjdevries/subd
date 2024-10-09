use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use subd_types::ElevenLabsRequest;
use subd_types::Event;
use subd_types::TransformOBSTextRequest;
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

pub fn twitch_chat_filename(username: &str, voice: &str) -> String {
    let now: DateTime<Utc> = Utc::now();

    format!("{}_{}_{}", now.timestamp(), username, voice)
}

pub fn chop_text(starting_text: &str) -> String {
    // TODO: Is it possible to remove this?
    let mut seal_text = starting_text.to_string();

    let spaces: Vec<_> = starting_text.match_indices(' ').collect();
    let line_length_modifier = 20;
    let mut line_length_limit = 20;
    for val in spaces.iter() {
        if val.0 > line_length_limit {
            seal_text.replace_range(val.0..=val.0, "\n");
            line_length_limit += line_length_modifier;
        }
    }

    seal_text
}

pub async fn set_voice(
    voice: &str,
    username: &str,
    pool: &sqlx::PgPool,
) -> Result<()> {
    let model = stream_character::user_stream_character_information::Model {
        username: username.to_string(),
        voice: voice.to_string().to_lowercase(),
        obs_character: subd_types::consts::get_default_stream_character_source(
        ),
        random: false,
    };

    model.save(pool).await?;

    Ok(())
}

pub async fn talk_in_voice(
    contents: &str,
    voice: &str,
    username: &str,
    tx: &broadcast::Sender<Event>,
) -> Result<()> {
    let spoken_string = contents.replace(&format!("!voice {}", &voice), "");

    if spoken_string.is_empty() {
        return Ok(());
    }

    let seal_text = chop_text(&spoken_string);

    let voice_text = spoken_string.clone();
    let _ = tx.send(Event::ElevenLabsRequest(ElevenLabsRequest {
        voice: Some(voice.to_string()),
        message: seal_text,
        voice_text,
        username: username.to_string(),
        ..Default::default()
    }));
    Ok(())
}

pub async fn use_random_voice(
    contents: &str,
    username: &str,
    tx: &broadcast::Sender<Event>,
) -> Result<()> {
    let voices_contents = fs::read_to_string("data/voices.json").unwrap();
    let voices: Vec<Voice> = serde_json::from_str(&voices_contents).unwrap();
    let mut rng = thread_rng();
    let random_index = rng.gen_range(0..voices.len());
    let random_voice = &voices[random_index];

    let spoken_string = contents.replace("!random", "");
    let speech_bubble_text = chop_text(&spoken_string);
    let voice_text = spoken_string;

    let _ = tx.send(Event::TransformOBSTextRequest(TransformOBSTextRequest {
        message: random_voice.name.to_string(),

        // TODO: This should probably be a different Text Source
        text_source: "Soundboard-Text".to_string(),
    }));

    let _ = tx.send(Event::ElevenLabsRequest(ElevenLabsRequest {
        voice: Some(random_voice.name.to_string()),
        message: speech_bubble_text,
        voice_text,
        username: username.to_string(),
        ..Default::default()
    }));
    Ok(())
}

pub async fn build_stream_character(
    pool: &sqlx::PgPool,
    username: &str,
) -> Result<StreamCharacter> {
    let default_voice = subd_types::consts::get_twitch_default_voice();

    let voice = match stream_character::get_voice_from_username(pool, username)
        .await
    {
        Ok(voice) => voice,
        Err(_) => {
            println!("No Voice Found, Using Default");

            return Ok(StreamCharacter {
                username: username.to_string(),
                voice: Some(default_voice.to_string()),
                source: subd_types::consts::get_default_stream_character_source(
                ),
            });
        }
    };

    let character = subd_types::consts::get_default_stream_character_source();

    Ok(StreamCharacter {
        username: username.to_string(),
        voice: Some(voice.to_string()),
        source: character.to_string(),
    })
}

// ============= //
// Audio Effects //
// ============= //

fn add_postfix_to_filepath(filepath: &str, postfix: &str) -> String {
    match filepath.rfind('.') {
        Some(index) => {
            let path = filepath[..index].to_string();
            let filename = filepath[index..].to_string();
            format!("{}{}{}", path, postfix, filename)
        }
        None => filepath.to_string(),
    }
}

pub fn normalize_tts_file(local_audio_path: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_norm");
    let ffmpeg_status = Command::new("ffmpeg")
        .args(["-i", local_audio_path, &audio_dest_path])
        .status()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_status.success() {
        Ok(audio_dest_path)
    } else {
        println!("Failed to normalize audio");
        Ok(local_audio_path.to_string())
    }
}

pub fn stretch_audio(local_audio_path: &str, stretch: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_stretch");
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
            &audio_dest_path,
            "stretch",
            stretch,
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

pub fn change_pitch(local_audio_path: &str, pitch: &str) -> Result<String> {
    let postfix = format!("{}_{}", "_pitch", pitch);
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, &postfix);
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
            &audio_dest_path,
            "pitch",
            pitch,
        ])
        .status()
        .expect("Failed to execute sox");

    Ok(audio_dest_path)
}

pub fn add_reverb(local_audio_path: &str) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(local_audio_path, "_reverb");
    Command::new("sox")
        .args([
            "-t",
            "wav",
            local_audio_path,
            &audio_dest_path,
            "gain",
            "-2",
            "reverb",
            "70",
            "100",
            "50",
            "100",
            "10",
            "2",
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

// ================= //
// Finding Functions //
// ================= //

pub fn find_voice_id_by_name(name: &str) -> Option<(String, String)> {
    // We should replace this with an API call
    // or call it every once-in-a-while and "cache"
    let data =
        fs::read_to_string("data/voices.json").expect("Unable to read file");
    let voice_list: VoiceList =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    // Why is this not his voice
    //

    let name_lowercase = name.to_lowercase();

    for voice in voice_list.voices {
        if voice.name.to_lowercase() == name_lowercase {
            println!("Using ID {} for Voice: {}", voice.voice_id, name);
            return Some((voice.voice_id, voice.name));
        }
    }
    None
}

pub fn sanitize_chat_message(raw_msg: &str) -> String {
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

pub fn find_random_voice() -> (String, String) {
    let data =
        fs::read_to_string("data/voices.json").expect("Unable to read file");

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
