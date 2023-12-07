use crate::music_scenes;
use crate::obs;
use crate::obs_source;
use crate::twitch_stream_state;
use crate::uberduck;
use anyhow::Result;
use dotenv::dotenv;
use obws::Client as OBSClient;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Error};
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

#[derive(Serialize)]
struct CloneVoiceRequest {
    name: String,
    description: String,
    files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Voice {
    voice_id: String,
    name: String,
    // other fields...
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

            let transform_settings = &splitmsg.get(1).unwrap();
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

            let transform_settings = &splitmsg.get(1).unwrap();
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
            let stretch = &splitmsg.get(1).unwrap();
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
            let pitch = &splitmsg.get(1).unwrap();
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
            let default_voice = obs::TWITCH_DEFAULT_VOICE.to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            uberduck::set_voice(
                voice.to_string(),
                msg.user_name.to_string(),
                pool,
            )
            .await
        }

        "!voice" => {
            let default_voice = obs::TWITCH_DEFAULT_VOICE.to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            uberduck::talk_in_voice(
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

            let music_list: Vec<&str> = music_scenes::VOICE_TO_MUSIC
                .iter()
                .map(|(_, scene)| scene.music)
                .collect();
            for source in music_list.iter() {
                let _ = obs_source::hide_source(
                    background_scene,
                    source,
                    obs_client,
                )
                .await;
            }

            let filter_name = "3D Transform";
            let filter_enabled = obws::requests::filters::SetEnabled {
                // TODO: Find the const
                source: "begin",
                filter: &filter_name,
                enabled: false,
            };
            obs_client.filters().set_enabled(filter_enabled).await?;
            // Disable Global Voice Mode
            twitch_stream_state::turn_off_global_voice(&pool).await?;
            Ok(())
        }

        "!random" => {
            Ok(())
            // uberduck::use_random_voice(msg.contents.clone(), msg.user_name, tx)
            //     .await
        }

        // "!my_voice" | "!myvoice" | "!my_name" | "!myname" => {
        //     // let voice = msg.voice.to_string();
        //    let info = format!("{} - {}", msg.user_name, voice);
        //    send_message(twitch_client, info).await?;
        //     Ok(())
        // }

        // TODO: move this somewhere more apporpriate
        "!no_dalle_mode" => {
            if not_beginbot {
                return Ok(());
            }
            twitch_stream_state::turn_off_dalle_mode(&pool).await?;
            Ok(())
        }

        "!dalle_mode" => {
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

        // !clone_no_dl name
        "!clone_no_dl" => {
            if not_beginbot {
                return Ok(());
            }

            if splitmsg.len() == 2 {
                println!("We are going for it");

                let name = &splitmsg[1];

                let mut mp3s: HashSet<String> = vec![].into_iter().collect();

                let split_mp3_folder =
                    format!("/home/begin/code/subd/tmp/cloned/split_{}/", name);
                let soundeffect_files = fs::read_dir(split_mp3_folder).unwrap();
                for split_file in soundeffect_files {
                    // we can filter by
                    mp3s.insert(
                        split_file.unwrap().path().display().to_string(),
                    );
                }

                let vec: Vec<String> = mp3s.into_iter().collect();
                let voice_clone = VoiceClone {
                    name: name.to_string(),
                    description: "a cloned voice".to_string(),
                    files: vec,
                    labels: HashMap::new(),
                };

                println!("About to from_clone");

                let api_base_url_v1 = "https://api.elevenlabs.io/v1";
                let result = from_clone(voice_clone, api_base_url_v1).await;
                println!("Result: {:?}", result);

                // We want the ID to not be escapae
                let voice_id = result.unwrap();
                println!("Result: {:?}", voice_id);

                // We save the JSON
                let filename = format!("/home/begin/code/subd/voices.json");
                let mut file = fs::File::open(filename.clone()).unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data)?;

                let mut root: VoiceRoot = serde_json::from_str(&data)?;

                let new_voice = Voice {
                    voice_id,
                    name: name.to_string(),
                    // Initialize other fields as needed...
                };

                // Add the new entry to the 'voices' vector
                root.voices.push(new_voice);

                // Serialize the modified object back to a JSON string
                let modified_json = serde_json::to_string_pretty(&root)?;

                // Write the modified JSON back to the file
                let mut file = fs::File::create(filename)?;
                file.write_all(modified_json.as_bytes())?;

                // let client = Client::new();
                // If I update the voices.json, it will work
                // let audio_response = client.post("https://api.elevenlabs.io/generate")
                //     .bearer_auth(&api_key)
                //     .json(&json!({
                //         "text": "Hi! I'm a cloned voice!",
                //         "voice": cloned_voice  , // Assuming voice contains the necessary identifier
                //     }))
                //     .send()
                //     .await
                //     .expect("Failed to send request");
            }

            Ok(())
        }

        // !clone name URL URL URL
        "!clone" => {
            dotenv().ok();
            if not_beginbot {
                return Ok(());
            }

            if splitmsg.len() < 3 {
                // we need at least a name and URL
                return Ok(());
            }

            // So we need to iterate over everything 2 on
            if splitmsg.len() > 2 {
                let name = &splitmsg[1];
                let urls = &splitmsg[2..];

                // Create the directory for the split sounds
                let split_mp3_folder =
                    format!("/home/begin/code/subd/tmp/cloned/split_{}/", name);
                match fs::create_dir(&split_mp3_folder) {
                    Ok(_) => println!("Directory created successfully"),
                    Err(e) => println!("Error creating directory: {:?}", e),
                }

                // // So we need to actually get all the files matching a pattern
                for (index, url) in urls.iter().enumerate() {
                    println!("{index} URL {url}");
                    let cloned_mp3 = format!(
                        "/home/begin/code/subd/tmp/cloned/{}-{}.wav",
                        name, index
                    );
                    let _ffmpeg_status = Command::new("yt-dlp")
                        .args(&[
                            "-x",
                            "--audio-format",
                            "wav",
                            &url,
                            "-o",
                            &cloned_mp3,
                        ])
                        .status()
                        .expect("Failed to execute ffmpeg");

                    // ffmpeg -i kim-1.wav -f segment -segment_time 50 -c copy split_kim/kim-1-%d.wav
                    let split_file_name = format!(
                        "{}{}-{}-%d.wav",
                        split_mp3_folder, name, index
                    );
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
                }

                let soundeffect_files = fs::read_dir(split_mp3_folder).unwrap();
                let mut mp3s: HashSet<String> = vec![].into_iter().collect();
                for split_file in soundeffect_files {
                    mp3s.insert(
                        split_file.unwrap().path().display().to_string(),
                    );
                }

                let vec: Vec<String> = mp3s.into_iter().collect();
                let voice_clone = VoiceClone {
                    name: name.to_string(),
                    description: "a cloned voice".to_string(),
                    files: vec,
                    labels: HashMap::new(),
                };

                let api_base_url_v1 = "https://api.elevenlabs.io/v1";
                let result = from_clone(voice_clone, api_base_url_v1).await;
                println!("Result: {:?}", result);

                // We want the ID to not be escapae
                let voice_id = result.unwrap();
                println!("Result: {:?}", voice_id);

                // We save the JSON
                let filename = format!("/home/begin/code/subd/voices.json");
                let mut file = fs::File::open(filename.clone()).unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data)?;

                let mut root: VoiceRoot = serde_json::from_str(&data)?;

                let new_voice = Voice {
                    voice_id,
                    name: name.to_string(),
                    // Initialize other fields as needed...
                };

                // Add the new entry to the 'voices' vector
                root.voices.push(new_voice);

                // Serialize the modified object back to a JSON string
                let modified_json = serde_json::to_string_pretty(&root)?;

                // Write the modified JSON back to the file
                let mut file = fs::File::create(filename)?;
                file.write_all(modified_json.as_bytes())?;

                // let client = Client::new();
                // If I update the voices.json, it will work
                // let audio_response = client.post("https://api.elevenlabs.io/generate")
                //     .bearer_auth(&api_key)
                //     .json(&json!({
                //         "text": "Hi! I'm a cloned voice!",
                //         "voice": cloned_voice  , // Assuming voice contains the necessary identifier
                //     }))
                //     .send()
                //     .await
                //     .expect("Failed to send request");
            }

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

async fn from_clone(
    voice_clone: VoiceClone,
    api_base_url_v1: &str,
) -> Result<String, Error> {
    let api_key = env::var("ELEVENLABS_API_KEY")
        .expect("Expected ELEVENLABS_API_KEY in .env");
    let client = Client::new();
    let url = format!("{}/voices/add", api_base_url_v1);

    let mut form = Form::new()
        .text("name", voice_clone.name)
        .text("description", voice_clone.description)
        .text(
            "labels",
            serde_json::to_string(&voice_clone.labels).unwrap(),
        );

    for file_path in voice_clone.files {
        let path = Path::new(&file_path);
        let mut file = File::open(path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
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

    let status = response.status();
    if status.is_success() {
        let voice_id: String = response
            .json::<serde_json::Value>()
            .await?
            .get("voice_id")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        Ok(voice_id)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}

// voice_id='45XeoJwbLhXJHjCXAi7q' name='owen' category='cloned' description='wow a cool guy' labels={} samples=[VoiceSample(sample_id='MtUrilOLqpnJeqxFuSn4', file_name='owen_140136286216160.mp3', mime_type='audio/mpeg', size_bytes=7167501, hash='e8130f475e206a3afb98443880a75aa4')] design=None preview_url=None settings=VoiceSettings(stability=0.5, similarity_boost=0.75, style=0.0, use_speaker_boost=True)
//
// #[derive(Debug, Deserialize)]
// struct VoiceSample {
//     sample_id: String,
//     file_name: String,
//     mime_type: String,
//     size_bytes: u64,
//     hash: String,
// }
//
// #[derive(Debug, Deserialize)]
// struct VoiceSettings {
//     stability: f32,
//     similarity_boost: f32,
//     style: f32,
//     use_speaker_boost: bool,
// }
//
// #[derive(Debug, Deserialize)]
// struct VoiceResponse {
//     voice_id: String,
//     name: String,
//     category: String,
//     description: String,
//     labels: serde_json::Value, // Using Value to accommodate an unspecified structure
//     samples: Vec<VoiceSample>,
//     design: Option<serde_json::Value>, // Assuming this can be null
//     preview_url: Option<String>,
//     settings: VoiceSettings,
// }
