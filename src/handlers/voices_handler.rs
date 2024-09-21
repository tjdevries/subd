use crate::constants;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use obs_service::obs_source;
use subd_elevenlabs;
use twitch_stream_state;
// use dotenv::dotenv;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;

pub struct VoicesHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
}

// #[derive(Serialize)]
//struct CloneVoiceRequest {
//    name: String,
//    description: String,
//    files: Vec<String>,
//}

#[derive(Serialize, Deserialize, Debug)]
struct Voice {
    voice_id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct VoiceRoot {
    voices: Vec<Voice>,
}

#[derive(Serialize, Deserialize)]
struct VoiceClone {
    name: String,
    description: String,
    labels: HashMap<String, String>,
    files: Vec<String>, // Vec of file paths
}

#[async_trait]
impl EventHandler for VoicesHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };
            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match handle_voices_commands(
                &tx,
                &self.obs_client,
                &self.pool,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error with handle_voices_command: {err}");
                    continue;
                }
            }
        }
    }
}

pub async fn handle_voices_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let is_mod = msg.roles.is_twitch_mod();
    let is_vip = msg.roles.is_twitch_vip();
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let background_scene = "BackgroundMusic";

    let command = splitmsg[0].as_str();

    match command {
        // play file.xxx echos 0.8 0.7 700.0 0.25 700.0 0.3
        // The sample will be bounced twice in asymmetric echos:
        // play file.xxx echos 0.8 0.7 700.0 0.25 900.0 0.3
        // The sample will sound as if played in a garage:
        // play file.xxx echos 0.8 0.7 40.0 0.25 63.0 0.3
        "!echo" => {
            if splitmsg.len() < 2 {
                return Ok(());
            }

            let transform_settings =
                splitmsg.get(1).ok_or(anyhow!("No settings to transform"))?;
            let contents = &splitmsg[2..].join(" ");
            let word_count = &splitmsg[2..].len();

            let (pitch, stretch, reverb) =
                parse_transform_settings(transform_settings, *word_count);
            let pitch = format!("{}", pitch);
            let stretch = format!("{}", stretch);
            println!("{} {} {}", pitch, stretch, reverb);

            let _ = tx.send(Event::ElevenLabsRequest(
                subd_types::ElevenLabsRequest {
                    source: Some("begin".to_string()),
                    message: contents.to_string(),
                    username: msg.user_name.to_string(),
                    pitch: Some(pitch),
                    stretch: Some(stretch),
                    reverb,

                    ..Default::default()
                },
            ));
            Ok(())
        }

        // !transform PITCH/STRETCH/REVERB
        "!transform" => {
            if splitmsg.len() < 2 {
                return Ok(());
            }

            // How do you go from Option to Result,
            let transform_settings =
                splitmsg.get(1).ok_or(anyhow!("No settings to transform"))?;
            let contents = &splitmsg[2..].join(" ");
            let word_count = &splitmsg[2..].len();

            let (pitch, stretch, reverb) =
                parse_transform_settings(transform_settings, *word_count);
            let pitch = format!("{}", pitch);
            let stretch = format!("{}", stretch);
            println!("{} {} {}", pitch, stretch, reverb);

            let _ = tx.send(Event::ElevenLabsRequest(
                subd_types::ElevenLabsRequest {
                    source: Some("begin".to_string()),
                    message: contents.to_string(),
                    username: msg.user_name.to_string(),
                    pitch: Some(pitch),
                    stretch: Some(stretch),
                    reverb,

                    ..Default::default()
                },
            ));
            Ok(())
        }

        "!stretch" => {
            let stretch =
                &splitmsg.get(1).ok_or(anyhow!("Nothing to !stretch"))?;
            let contents = &splitmsg[2..].join(" ");

            let _ = tx.send(Event::ElevenLabsRequest(
                subd_types::ElevenLabsRequest {
                    source: Some("begin".to_string()),

                    message: contents.to_string(),
                    username: msg.user_name.to_string(),

                    stretch: Some(stretch.to_string()),
                    ..Default::default()
                },
            ));
            Ok(())
        }

        "!pitch" => {
            let pitch = &splitmsg.get(1).ok_or(anyhow!("Nothing to !pitch"))?;
            let contents = &splitmsg[2..].join(" ");

            let _ = tx.send(Event::ElevenLabsRequest(
                subd_types::ElevenLabsRequest {
                    source: Some("begin".to_string()),
                    // What is the message voice_text?
                    message: contents.to_string(),
                    username: msg.user_name.to_string(),
                    pitch: Some(pitch.to_string()),

                    ..Default::default()
                },
            ));

            Ok(())
        }

        "!reverb" => {
            let contents = &splitmsg[1..].join(" ");

            let _ = tx.send(Event::ElevenLabsRequest(
                subd_types::ElevenLabsRequest {
                    source: Some("begin".to_string()),
                    message: contents.to_string(),
                    username: msg.user_name.to_string(),
                    reverb: true,
                    ..Default::default()
                },
            ));

            Ok(())
        }

        "!set_voice" | "!setvoice" | "!set_name" | "!setname" => {
            let default_voice = subd_types::consts::get_twitch_default_source();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            subd_elevenlabs::set_voice(
                voice.to_string(),
                msg.user_name.to_string(),
                pool,
            )
            .await
        }

        "!voice" => {
            let default_voice = subd_types::consts::get_twitch_default_source();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            subd_elevenlabs::talk_in_voice(
                msg.contents.clone(),
                voice.to_string(),
                msg.user_name,
                tx,
            )
            .await
        }

        // ===========================================
        // == Voices
        // ===========================================
        "!nothing" => {
            if !is_mod && !is_vip {
                return Ok(());
            }
            println!("Time for !nothing");

            let music_list: Vec<&str> = constants::VOICE_TO_MUSIC
                .iter()
                .map(|(_, scene)| scene.music)
                .collect();
            for source in music_list.iter() {
                if let Err(e) = obs_source::hide_source(
                    background_scene,
                    source,
                    obs_client,
                )
                .await
                {
                    eprintln!(
                        "Error hiding source: {} {} | {:?}",
                        background_scene, source, e
                    );
                };
            }

            // Disable Global Voice Mode
            twitch_stream_state::turn_off_global_voice(&pool).await
        }

        "!random" => {
            Ok(())
            // elevenlabs::use_random_voice(msg.contents.clone(), msg.user_name, tx)
            //     .await
        }

        // "!my_voice" | "!myvoice" | "!my_name" | "!myname" => {
        //     // let voice = msg.voice.to_string();
        //    let info = format!("{} - {}", msg.user_name, voice);
        //    send_message(twitch_client, info).await?;
        //     Ok(())
        // }

        // TODO: move this somewhere more apporpriate
        "!disable_stable_diffusion" => {
            if not_beginbot {
                return Ok(());
            }
            twitch_stream_state::disable_stable_diffusion(&pool).await?;
            Ok(())
        }

        "!enable_stable_diffusion" => {
            if not_beginbot {
                return Ok(());
            }
            println!("Turning on Dalle Mode");
            twitch_stream_state::enable_stable_diffusion(&pool).await?;
            Ok(())
        }

        // TODO: move this somewhere more apporpriate
        "!disable_dalle" => {
            if not_beginbot {
                return Ok(());
            }
            twitch_stream_state::turn_off_dalle_mode(&pool).await?;
            Ok(())
        }

        "!enable_dalle" => {
            if not_beginbot {
                return Ok(());
            }
            println!("Turning on Dalle Mode");
            twitch_stream_state::turn_on_dalle_mode(&pool).await?;
            Ok(())
        }

        "!no_global_voice" => {
            if not_beginbot {
                return Ok(());
            }

            twitch_stream_state::turn_off_global_voice(&pool).await?;
            Ok(())
        }

        "!global_voice" => {
            if not_beginbot {
                return Ok(());
            }

            println!("Turning on Global Voice");
            twitch_stream_state::turn_on_global_voice(&pool).await?;

            Ok(())
        }

        "!isolation" => {
            // I need an audio file path
            let api_base_url_v1 = "https://api.elevenlabs.io/v1";

            let url = &splitmsg[1];
            let name = "test";
            let index = 0;
            let audio_file_path =
                download_with_yt_dlp(name, url, index).await?;

            // let cropped_audio = crop_audio(&audio_file_path, 4).await?;
            isolate_voice(api_base_url_v1, &audio_file_path).await?;
            Ok(())
        }

        // !clone_no_dl name
        "!clone_no_dl" => {
            if not_beginbot {
                return Ok(());
            }

            // We need to at least have a name
            if splitmsg.len() == 2 {
                let name = &splitmsg[1];

                let split_mp3_folder = format!("./tmp/cloned/split_{}/", name);

                let vec_of_mp3s =
                    load_folder_into_vec_of_mp3s(&split_mp3_folder).await?;

                let voice_clone = VoiceClone {
                    name: name.to_string(),
                    description: "a cloned voice".to_string(),
                    files: vec_of_mp3s,
                    labels: HashMap::new(),
                };

                let api_base_url_v1 = "https://api.elevenlabs.io/v1";
                let result = from_clone(voice_clone, api_base_url_v1).await;
                println!("Result: {:?}", result);

                let voice_id = result?;
                println!("Result: {:?}", voice_id);

                save_voice_id_to_voices_json(name, voice_id).await?;
            }

            Ok(())
        }

        "!clone" => {
            if not_beginbot {
                return Ok(());
            }

            // we need at least a Name and URL
            if splitmsg.len() < 3 {
                return Ok(());
            }

            let name = &splitmsg[1];
            let urls = &splitmsg[2..];

            // Create the directory for the split sounds
            let split_mp3_folder = format!("./tmp/cloned/split_{}/", name);
            match fs::create_dir(&split_mp3_folder) {
                Ok(_) => println!("Directory created successfully"),
                Err(e) => println!("Error creating directory: {:?}", e),
            }

            for (index, url) in urls.iter().enumerate() {
                // We don't want to send more than 15 samples to Elevenlabs
                if index > 15 {
                    continue;
                }
                let cloned_mp3 = download_with_yt_dlp(name, url, index).await?;

                // This is a hack to just try and clone troy
                // let cloned_mp3 = "troy.wav";
                split_song(name, &cloned_mp3, &split_mp3_folder, index).await?;
            }

            let voice_id =
                clone_voice_with_elevenlabs(name, &split_mp3_folder).await?;
            println!("Voice ID: {}", voice_id);

            save_voice_id_to_voices_json(name, voice_id).await?;

            Ok(())
        }

        _ => Ok(()),
    }
}

fn parse_transform_settings(
    transform_settings: &str,
    word_count: usize,
) -> (i32, f32, bool) {
    let mut pitch: i32 = 0;
    let mut stretch: f32 = 1.0;
    let mut reverb: bool = false;

    let settings: Vec<&str> = transform_settings.split('/').collect();

    if settings.len() > 0 {
        if let Ok(parsed_pitch) = settings[0].parse::<i32>() {
            if parsed_pitch >= -1000 && parsed_pitch <= 1000 {
                pitch = parsed_pitch;
            }
        }
    }

    let stretch_limit = if word_count > 2 { 3.0 } else { 10.0 };
    if settings.len() > 1 {
        if let Ok(parsed_stretch) = settings[1].parse::<f32>() {
            if parsed_stretch >= 0.1 && parsed_stretch <= stretch_limit {
                stretch = parsed_stretch;
            }
        }
    }

    if settings.len() > 2 {
        reverb = matches!(settings[2].to_lowercase().as_str(), "true" | "t");
    }

    (pitch, stretch, reverb)
}

async fn isolate_voice(
    api_base_url_v1: &str,
    audio_file_path: &str,
) -> Result<()> {
    let api_key = env::var("ELEVENLABS_API_KEY")
        .expect("Expected ELEVENLABS_API_KEY in .env");
    let client = Client::new();
    let url = format!("{}/audio-isolation", api_base_url_v1);

    let file_name = Path::new(audio_file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid file name"))?;

    // let part = Part::file(audio_file_path).await?;
    let file_bytes = fs::read(audio_file_path)?;
    let part = Part::bytes(file_bytes)
        .file_name(file_name.to_string())
        .mime_str("audio/mpeg")?; // Adjust the MIME type if needed
    let form = Form::new()
        .part("audio", part)
        .text("model_id", "eleven_monolingual_v1");

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let output_path = format!("isolated_{}", file_name);
        let mut output_file = File::create(output_path)?;
        output_file.write_all(&bytes)?;
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to isolate voice: {:?} {}",
            response.status(),
            response.text().await?
        ))
    }
}

async fn from_clone(
    voice_clone: VoiceClone,
    api_base_url_v1: &str,
) -> Result<String> {
    let api_key = env::var("ELEVENLABS_API_KEY")
        .expect("Expected ELEVENLABS_API_KEY in .env");
    let client = Client::new();
    let url = format!("{}/voices/add", api_base_url_v1);

    let mut form = Form::new()
        .text("name", voice_clone.name)
        .text("description", voice_clone.description)
        .text("labels", serde_json::to_string(&voice_clone.labels)?);

    for file_path in voice_clone.files {
        let path = Path::new(&file_path);
        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        let part = Part::bytes(contents)
            .file_name(file_path)
            .mime_str("audio/mpeg")?;
        form = form.part("files", part);
    }

    let response = client
        .post(url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    response
        .json::<serde_json::Value>()
        .await?
        .get("voice_id")
        .ok_or(anyhow!("Couldn't find voice_id in response"))?
        .as_str()
        .ok_or(anyhow!("Couldn't convert voice_id to str"))
        .map(|s| s.to_string())
}

async fn save_voice_id_to_voices_json(
    name: &str,
    voice_id: String,
) -> Result<()> {
    let filename = format!("./data/voices.json");
    let mut file = fs::File::open(filename.clone())?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let mut root: VoiceRoot = serde_json::from_str(&data)?;
    let new_voice = Voice {
        voice_id,
        name: name.to_string(),
    };
    root.voices.push(new_voice);
    let modified_json = serde_json::to_string_pretty(&root)?;
    let mut file = fs::File::create(filename)?;
    file.write_all(modified_json.as_bytes())?;
    Ok(())
}

async fn split_song(
    name: &str,
    cloned_mp3: &str,
    split_mp3_folder: &str,
    index: usize,
) -> Result<()> {
    let split_file_name =
        format!("{}{}-{}-%d.wav", split_mp3_folder, name, index);
    let _ffmpeg_status = Command::new("ffmpeg")
        .args(&[
            "-i",
            &cloned_mp3,
            "-f",
            "segment",
            "-segment_time",
            "50",
            "-c",
            "copy",
            &split_file_name,
        ])
        .status()
        .expect("Failed to execute ffmpeg");
    Ok(())
}

async fn download_with_yt_dlp(
    name: &str,
    url: &str,
    index: usize,
) -> Result<String> {
    // not sure if we need an index here
    println!("{index} URL {url}");
    let cloned_mp3 = format!("./tmp/cloned/{}-{}.wav", name, index);
    let _ffmpeg_status = Command::new("yt-dlp")
        .args(&["-x", "--audio-format", "wav", &url, "-o", &cloned_mp3])
        .status()
        .expect("Failed to execute ffmpeg");
    Ok(cloned_mp3)
}

async fn _crop_audio(input_file: &str, crop_time: usize) -> Result<String> {
    let output_file =
        format!("{}_cropped.wav", input_file.trim_end_matches(".wav"));
    let _ffmpeg_status = Command::new("ffmpeg")
        .args(&[
            "-i",
            input_file,
            "-t",
            &crop_time.to_string(),
            "-acodec",
            "copy",
            &output_file,
        ])
        .status()
        .expect("Failed to execute ffmpeg");
    Ok(output_file)
}

async fn load_folder_into_vec_of_mp3s(
    split_mp3_folder: &str,
) -> Result<Vec<String>> {
    let soundeffect_files = fs::read_dir(split_mp3_folder)?;
    let mut mp3s: HashSet<String> = vec![].into_iter().collect();
    for split_file in soundeffect_files {
        mp3s.insert(split_file?.path().display().to_string());
    }
    let vec_of_mp3s: Vec<String> = mp3s.into_iter().collect();
    Ok(vec_of_mp3s)
}

async fn clone_voice_with_elevenlabs(
    name: &str,
    split_mp3_folder: &str,
) -> Result<String> {
    let vec_of_mp3s = load_folder_into_vec_of_mp3s(&split_mp3_folder).await?;

    let voice_clone = VoiceClone {
        name: name.to_string(),
        description: "a cloned voice".to_string(),
        files: vec_of_mp3s,
        labels: HashMap::new(),
    };
    let api_base_url_v1 = "https://api.elevenlabs.io/v1";
    let result = from_clone(voice_clone, api_base_url_v1).await?;
    Ok(result)
}
