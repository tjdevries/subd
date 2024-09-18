// use crate::ai_images::image_generation::GenerateImage;
use crate::ai_scene;
use crate::audio;
// use crate::openai::dalle;
use crate::redirect;
use crate::stream_character;
use crate::twitch_stream_state;
use ai_friends;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use events::EventHandler;
use fal_ai;
use obws::Client as OBSClient;
use rand::{seq::SliceRandom, thread_rng};
use rodio::*;
// use stable_diffusion::models::GenerateAndArchiveRequest;
// use stable_diffusion::models::RequestType;
// use stable_diffusion::run_from_prompt;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::sync::Arc;
use subd_types::AiScenesRequest;
use subd_types::Event;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use twitch_chat::client::send_message;

use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

// ============================================================
//
// Should this have an OBS Client as well
pub struct AiScenesHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pub elevenlabs: Elevenlabs,
    pub obs_client: OBSClient,
}

async fn build_face_scene_request(voice: String) -> Result<Option<String>> {
    let voice_to_face_image = HashMap::from([
        ("satan".to_string(), "satan.png".to_string()),
        ("god".to_string(), "god.png".to_string()),
        ("ethan".to_string(), "alex_jones.png".to_string()),
        // We need a systen for multiple photos
        // ("teej".to_string(), "teej.png".to_string()),
        // ("teej".to_string(), "teej_2.jpg".to_string()),
        ("prime".to_string(), "green_prime.png".to_string()),
        ("teej".to_string(), "teej_3.png".to_string()),
        ("melkey".to_string(), "melkey.png".to_string()),
    ]);
    Ok(voice_to_face_image.get(&voice).cloned())
}

async fn run_ai_scene(
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
    if let Some(prompt) = &ai_scene_req.prompt {
        twitch_stream_state::set_ai_background_theme(&pool, &prompt).await?;
    };

    let twitch_client = Arc::new(Mutex::new(twitch_client));
    let clone_twitch_client = twitch_client.clone();
    let locked_twitch_client = clone_twitch_client.lock().await;
    let obs_client = Arc::new(Mutex::new(obs_client));
    let obs_client_clone = obs_client.clone();
    let locked_obs_client = obs_client_clone.lock().await;

    // This is a dumb way to decide if the scene is AI Friend or not
    // this only works because we are correlating a voice like prime, with asking prime a question
    // through channel points
    let face_image = build_face_scene_request(final_voice.clone()).await?;
    match face_image {
        Some(image_file_path) => {
            trigger_ai_friend(
                &locked_obs_client,
                &locked_twitch_client,
                ai_scene_req,
                image_file_path,
                local_audio_path,
                final_voice,
            )
            .await?
        }
        None => {
            trigger_movie_trailer(
                ai_scene_req,
                &locked_twitch_client,
                local_audio_path,
            )
            .await?
        }
    }

    return Ok(());
}

async fn trigger_ai_friend(
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    ai_scene_req: &AiScenesRequest,
    image_file_path: String,
    local_audio_path: String,
    friend_name: String,
) -> Result<()> {
    println!("Syncing Lips and Voice for Image: {:?}", image_file_path);

    match sync_lips_and_update(
        &image_file_path,
        &local_audio_path,
        &obs_client,
        friend_name,
    )
    .await
    {
        Ok(_) => {
            if let Some(music_bg) = &ai_scene_req.music_bg {
                let _ = send_message(&twitch_client, music_bg.clone()).await;
            }
        }
        Err(e) => {
            eprintln!("Error syncing lips and updating: {:?}", e);
        }
    }
    Ok(())
}

#[async_trait]
impl EventHandler for AiScenesHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        println!("Starting AI Scenes Handler");
        loop {
            let event = rx.recv().await?;
            let ai_scene_req = match event {
                Event::AiScenesRequest(msg) => msg,
                _ => continue,
            };

            // If this crashes we just want to loop again
            // and we expect the error to be printing
            let _ = run_ai_scene(
                &self.twitch_client,
                &self.obs_client,
                &self.pool,
                &self.elevenlabs,
                &ai_scene_req,
            )
            .await;
        }
    }
}

async fn sync_lips_and_update(
    fal_image_file_path: &str,
    fal_audio_file_path: &str,
    obs_client: &OBSClient,
    friend_name: String,
) -> Result<()> {
    let video_bytes = ai_friends::sync_lips_to_voice(
        fal_image_file_path,
        fal_audio_file_path,
    )
    .await?;

    // We are saving he video
    let video_path = format!("./ai_assets/{}.mp4", friend_name);
    match tokio::fs::write(&video_path, &video_bytes).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing video: {:?}", e);
            return Err(anyhow!("Error writing video: {:?}", e));
        }
    }
    println!("Video saved to {}", video_path);

    let scene = "AIFriends";
    // let source = friend_name;
    let _ = crate::obs::obs_source::set_enabled(
        scene,
        &friend_name,
        false,
        &obs_client,
    )
    .await;

    // Not sure if I have to wait ofr how long to wait
    sleep(Duration::from_millis(100)).await;

    let _ = crate::obs::obs_source::set_enabled(
        scene,
        &friend_name,
        true,
        &obs_client,
    )
    .await;
    return Ok(());
}

async fn find_image_modes(pool: sqlx::PgPool) -> Result<(bool, bool)> {
    let twitch_state = twitch_stream_state::get_twitch_state(&pool);
    Ok(match twitch_state.await {
        Ok(state) => (state.enable_stable_diffusion, state.dalle_mode),
        Err(err) => {
            eprintln!("Error fetching twitch_stream_state: {:?}", err);
            (false, false)
        }
    })
}

fn set_volume(voice: String, sink: &Sink) -> Result<()> {
    match voice.as_str() {
        "melkey" => sink.set_volume(1.0),
        "beginbot" => sink.set_volume(1.0),
        "evil_pokimane" => sink.set_volume(1.0),
        "satan" => sink.set_volume(0.7),
        "god" => sink.set_volume(0.7),
        _ => {
            sink.set_volume(0.5);
        }
    };
    Ok(())
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

    add_voice_modifiers(ai_scenes_request, voice, local_audio_path)
}

fn add_voice_modifiers(
    req: &AiScenesRequest,
    voice: String,
    mut local_audio_path: String,
) -> Result<String> {
    if req.reverb {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    match &req.stretch {
        Some(stretch) => {
            local_audio_path =
                normalize_tts_file(local_audio_path.clone()).unwrap();
            local_audio_path =
                stretch_audio(local_audio_path, stretch.to_owned())?;
        }
        None => {}
    }

    match &req.pitch {
        Some(pitch) => {
            local_audio_path = normalize_tts_file(local_audio_path.clone())?;
            local_audio_path =
                change_pitch(local_audio_path, pitch.to_owned())?;
        }
        None => {}
    }

    if voice == "evil_pokimane" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = change_pitch(local_audio_path, "-200".to_string())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    if voice == "satan" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = change_pitch(local_audio_path, "-350".to_string())?;
        local_audio_path = add_reverb(local_audio_path.clone())?;
    }

    if voice == "god" {
        local_audio_path = normalize_tts_file(local_audio_path.clone())?;
        local_audio_path = add_reverb(local_audio_path)?;
    }

    return Ok(local_audio_path);
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
    let audio_dest_path =
        add_postfix_to_filepath(local_audio_path.clone(), "_norm".to_string());
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
    let audio_dest_path = add_postfix_to_filepath(
        local_audio_path.clone(),
        "_stretch".to_string(),
    );
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
            &audio_dest_path,
            "stretch",
            &stretch,
        ])
        .status()
        .expect("Failed to execute sox");
    Ok(audio_dest_path)
}

fn change_pitch(local_audio_path: String, pitch: String) -> Result<String> {
    let postfix = format!("{}_{}", "_pitch".to_string(), pitch);
    let audio_dest_path =
        add_postfix_to_filepath(local_audio_path.clone(), postfix);
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
            &audio_dest_path,
            "pitch",
            &pitch,
        ])
        .status()
        .expect("Failed to execute sox");

    Ok(audio_dest_path)
}

fn add_reverb(local_audio_path: String) -> Result<String> {
    let audio_dest_path = add_postfix_to_filepath(
        local_audio_path.clone(),
        "_reverb".to_string(),
    );
    Command::new("sox")
        .args(&[
            "-t",
            "wav",
            &local_audio_path,
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
    let voice_list: ai_scene::VoiceList =
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

    let voice_list: ai_scene::VoiceList =
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

async fn trigger_movie_trailer(
    ai_scene_req: &AiScenesRequest,
    locked_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    local_audio_path: String,
) -> Result<()> {
    if let Some(music_bg) = &ai_scene_req.music_bg {
        let _ = send_message(&locked_client, music_bg.clone()).await;
    }

    // We are supressing a whole bunch of alsa message
    let backup =
        redirect::redirect_stderr().expect("Failed to redirect stderr");

    let (_stream, stream_handle) =
        audio::get_output_stream("pulse").expect("stream handle");
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let file = BufReader::new(File::open(local_audio_path)?);
    sink.append(Decoder::new(BufReader::new(file))?);
    sink.sleep_until_end();
    redirect::restore_stderr(backup);
    return Ok(());
}
