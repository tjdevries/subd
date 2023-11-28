use anyhow::{bail, Result};
use std::collections::HashSet;
use crate::bootstrap;
use std::fs;
use std::borrow::Cow;
use std::path::Path;
use reqwest::{Body, Client, Error};
use std::io::{self, Read, Write};
use reqwest::multipart::{Form, Part};
use std::fs::File;
use rand::Rng;
use std::collections::HashMap;
use rand::seq::SliceRandom;
use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::move_transition_effects;
use crate::obs;
use crate::obs_combo;
use crate::obs_hotkeys;
use crate::dalle;
use crate::obs_scenes;
use crate::obs_source;
use crate::sdf_effects;
use crate::stream_character;
use crate::twitch_stream_state;
use crate::skybox;
use crate::uberduck;
use crate::music_scenes;
use crate::art_blocks;
use twitch_chat::send_message;
use obws::Client as OBSClient;
use obws::requests::scene_items::Scale;
use obws;
use std::process::Command;
use std::thread;
use std::time;
use subd_types::{Event, UserMessage, TransformOBSTextRequest};
use tokio::sync::broadcast;
use twitch_irc::{TwitchIRCClient, SecureTCPTransport, login::StaticLoginCredentials};

// This is forthe 
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use dotenv::dotenv;


// voice_id='45XeoJwbLhXJHjCXAi7q' name='owen' category='cloned' description='wow a cool guy' labels={} samples=[VoiceSample(sample_id='MtUrilOLqpnJeqxFuSn4', file_name='owen_140136286216160.mp3', mime_type='audio/mpeg', size_bytes=7167501, hash='e8130f475e206a3afb98443880a75aa4')] design=None preview_url=None settings=VoiceSettings(stability=0.5, similarity_boost=0.75, style=0.0, use_speaker_boost=True)
// 
 
#[derive(Debug, Deserialize)]
struct VoiceSample {
    sample_id: String,
    file_name: String,
    mime_type: String,
    size_bytes: u64,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct VoiceSettings {
    stability: f32,
    similarity_boost: f32,
    style: f32,
    use_speaker_boost: bool,
}

#[derive(Debug, Deserialize)]
struct VoiceResponse {
    voice_id: String,
    name: String,
    category: String,
    description: String,
    labels: serde_json::Value, // Using Value to accommodate an unspecified structure
    samples: Vec<VoiceSample>,
    design: Option<serde_json::Value>, // Assuming this can be null
    preview_url: Option<String>,
    settings: VoiceSettings,
}

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
pub async fn handle_obs_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let default_source = obs::DEFAULT_SOURCE.to_string();

    let is_mod = msg.roles.is_twitch_mod();
    let is_vip = msg.roles.is_twitch_vip();
    let background_scene = "BackgroundMusic";

    // We try and do some parsing on every command here
    // These may not always be what we want, but they are sensible
    // defaults used by many commands
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));

    let filter_value = splitmsg
        .get(3)
        .map_or(0.0, |x| x.trim().parse().unwrap_or(0.0));

    let scene = match obs_scenes::find_scene(source).await {
        Ok(scene) => scene.to_string(),
        Err(_) => obs::MEME_SCENE.to_string(),
    };
    
    let not_beginbot = msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    
   // This fails, and we stop
   // let voice = stream_character::get_voice_from_username(pool, &msg.user_name).await?;

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    //       we need to figure a way to look up the defaults per command
    //       because they could be different types

    println!("Splitmsg: {} | {}", splitmsg[0], msg.user_name);
    let command = splitmsg[0].as_str();
    let _ =  match command {

        // =================== //
        // === Experiments === //
        // =================== //
        "!wide" => {
            println!("Wide TIME!");

            let source = "begin";
            let filter_name = "3D-Transform-Orthographic";
            let duration = 5000;
            let filter_setting_name = "Scale.X";
            let filter_value = 300.0;
            let _ = move_transition_effects::trigger_move_value_3d_transform(
                source,
                filter_name,
                filter_setting_name,
                filter_value,
                duration,
                obs_client,
            ).await;
            Ok(())
        }

        "!normal" => {
            println!("Normal TIME!");

            // We need ways of duplicated settings
            let filter_name = "Default_3D-Transform-Perspective";
            let filter_enabled = obws::requests::filters::SetEnabled {
                // TODO: Find the const
                source: "begin",
                filter: &filter_name,
                enabled: true,
            };
            obs_client.filters().set_enabled(filter_enabled).await?;
            
            let filter_name = "Default_3D-Transform-Orthographic";
            let filter_enabled = obws::requests::filters::SetEnabled {
                source: "begin",
                filter: &filter_name,
                enabled: true,
            };
            obs_client.filters().set_enabled(filter_enabled).await?;
            
            let filter_name = "Default_3D-Transform-CornerPin";
            let filter_enabled = obws::requests::filters::SetEnabled {
                source: "begin",
                filter: &filter_name,
                enabled: true,
            };
            obs_client.filters().set_enabled(filter_enabled).await?;

            Ok(())

        }
        
        // This is a demonstration of updating a single Setting
        // We need to make sure it works going back forth for updating multi-effects
        "!nerd3" => {
            println!("Nerd TIME!");
 
            let source = "begin";
            let filter_name = "3D-Transform-Perspective";
            let duration = 5000;
            let filter_setting_name = "Rotation.X";
            let filter_value = -45.0;
            let _ = move_transition_effects::trigger_move_value_3d_transform(
                source,
                filter_name,
                filter_setting_name,
                filter_value,
                duration,
                obs_client,
            ).await;
            Ok(())
        }

        "!nerd" => {
            println!("Nerd TIME!");
 
            let source = "begin";
            let filter_name = "3D-Transform-Perspective";
            
            // See the settings aren't correct
            // We need to convert from the settings of the filter
            let new_settings = move_transition::MoveMultipleValuesSetting{
                filter: Some(filter_name.to_string()),
                scale_x: Some(125.3),
                scale_y: Some(140.6),
                position_y: Some(40.0),
                rotation_x: Some(-51.4),

                // Added this to test
                field_of_view: Some(90.0),
                ..Default::default()
            };

            let three_d_transform_filter_name = filter_name;
            let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
            
            _ = move_transition::update_and_trigger_move_values_filter(
                source,
                &move_transition_filter_name,
                duration,
                new_settings,
                &obs_client,
            )
            .await;

            Ok(())
        }

        "!chad" => {
            let source = "begin";
            let filter_name = "3D-Transform-Perspective";
        
            let new_settings = move_transition::MoveMultipleValuesSetting{
                filter: Some(filter_name.to_string()),
                scale_x: Some(217.0),
                scale_y: Some(200.0),
                rotation_x: Some(50.0),
                field_of_view: Some(108.0),
                move_value_type: 1,

                // If a previous Move_transition set this and you don't reset it, you're gonna hate you life
                position_y: Some(0.0),
                duration: Some(300),
                shear_x: Some(0.0),
                shear_y: Some(0.0),
                position_x: Some(0.0),
                rotation_y: Some(0.0),
                rotation_z: Some(0.0),
                ..Default::default()
            };

            // dbg!(&new_settings);
            let move_transition_filter_name = format!("Move_{}", filter_name);
            
            _ = move_transition::update_and_trigger_move_values_filter(
                source,
                &move_transition_filter_name,
                duration,
                new_settings,
                &obs_client,
            )
            .await;

            Ok(())
        }

        // ======================== //
        // === Rapper Functions === //
        // ======================== //

        "!reload_rapper" => {
            let source = "SpeechBubble";
            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                source,
                false,
                &obs_client,
            )
            .await;
            let ten_millis = time::Duration::from_millis(300);
            thread::sleep(ten_millis);
            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                source,
                true,
                &obs_client,
        )
        .await;
            Ok(())
        }
        // ===========================================
        // == Test Area
        // ===========================================
        
        "!durf" => {
            // Put any code you want to experiment w/ the chat with here
            Ok(())
        }

        // only Begin should be to do these sounds
        // Maybe Mods
        // ===========================================
        // == Stream State
        // ===========================================
        "!implicit" | "!peace" => {
            twitch_stream_state::update_implicit_soundeffects(&pool)
                .await?;
            Ok(())
        }
        "!explicit" => {
            twitch_stream_state::update_explicit_soundeffects(&pool)
                .await?;
            Ok(())
        }
        // returns the current state of stream
        "!state" => {
           let state = twitch_stream_state::get_twitch_state(&pool).await?;
           let msg = format!("Twitch State! {:?}", state);
           send_message(twitch_client, 
                msg).await?;
           // send_message(format!("Twitch State! {:?}", state));
            // twitch_stream_state::update_implicit_soundeffects(false, &pool)
            //     .await?;
            Ok(())
                
        }
        
        "!ab" => {
            println!("WE ARE INSIDE !ab: {}", msg.user_name);
            
            let ab_id = &splitmsg[1];
            let ab_url = format!("https://generator.artblocks.io/0x99a9b7c1116f9ceeb1652de04d5969cce509b069/{}", ab_id);

            let browser_settings = obws::requests::custom::source_settings::BrowserSource{
                url: ab_url.as_ref(),
                ..Default::default()
            };
            let set_settings = obws::requests::inputs::SetSettings{
                settings: &browser_settings,
                input: "AB-Browser",
                overlay: Some(true),
            };
            let _ = obs_client.inputs().set_settings(set_settings).await;
            
            Ok(())
            
        }
        
        "!chimera" => {
            let lower_bound = 233000000;
            let upper_bound = 233000986;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        
        
        "!watercolor" => {
            let lower_bound = 59000000;
            let upper_bound = 59000599;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        
        "!pig" => {
            let lower_bound = 129000000;
            let upper_bound = 129001023;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        

        "!run" => {
            let lower_bound = 138000000;
            let upper_bound = 138000999;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        
        
        "!vortex" | "!v" => {
            let lower_bound = 225000000;
            let upper_bound = 225000999;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        

        "!memories" | "!m" => {
            let lower_bound = 428000000;
            let upper_bound = 428000449;
            let contract = "0x99a9b7c1116f9ceeb1652de04d5969cce509b069";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        }
        
        "!steviep" | "!dopamine" | "!d" => {
            let lower_bound = 457000000;
            let upper_bound = 457000776;
            let contract = "0x99a9b7c1116f9ceeb1652de04d5969cce509b069";
            let _ = art_blocks::updates_ab_browser(&obs_client, contract.to_string(), lower_bound, upper_bound).await;
            Ok(())
        } 

        // ===========================================
        // == Voices
        // ===========================================

        "!nothing" => {
            if !is_mod && !is_vip {
                return Ok(());
            }
            
            let music_list: Vec<&str> = music_scenes::VOICE_TO_MUSIC.iter()
                .map(|(_, scene)| scene.music)
                .collect();
            for source in music_list.iter() {
                let _ = obs_source::hide_source(background_scene, source, obs_client).await;
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
            twitch_stream_state::turn_off_global_voice(&pool)
                .await?;
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

        // !global_voice Ethan
        "!no_global_voice" => {
            if not_beginbot {
                return Ok(())
            }
            
            twitch_stream_state::turn_off_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        // TODO: improve this 
        "!global_voice" => {
            if not_beginbot {
                return Ok(())
            }
            
            println!("Turning on Global Voice");
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            
            Ok(())
        }
        
        
        // !clone_no_dl name
        "!clone_no_dl" => {
            if not_beginbot {
                return Ok(())
            }
            
            if splitmsg.len() == 2 {
                 println!("We are going for it");
                
                let name = &splitmsg[1];
                
                let mut mp3s: HashSet<String> = vec![].into_iter().collect();
                
                let split_mp3_folder = format!("/home/begin/code/subd/tmp/cloned/split_{}/", name);
                let soundeffect_files = fs::read_dir(split_mp3_folder).unwrap();
                for split_file in soundeffect_files {

                // we can filter by 
                    mp3s.insert(split_file.unwrap().path().display().to_string());
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
            let api_key = env::var("ELEVENLABS_API_KEY").expect("Expected ELEVENLABS_API_KEY in .env");
            if not_beginbot {
                return Ok(())
            }
            
            if splitmsg.len() < 3 {
                // we need at least a name and URL
                return Ok(());
            }

            let mut clone_sample_urls_files: Vec<String> = vec![];
            
            // So we need to iterate over everything 2 on
            if splitmsg.len() > 2 {
                let name = &splitmsg[1];
                let urls = &splitmsg[2..];

                // Create the directory for the split sounds
                let split_mp3_folder = format!("/home/begin/code/subd/tmp/cloned/split_{}/", name);
                match fs::create_dir(&split_mp3_folder) {
                        Ok(_) => println!("Directory created successfully"),
                        Err(e) => println!("Error creating directory: {:?}", e),
                }
                    
                // // So we need to actually get all the files matching a pattern
                for (index, url) in urls.iter().enumerate() {
                    println!("{index} URL {url}");
                    let cloned_mp3 = format!("/home/begin/code/subd/tmp/cloned/{}-{}.wav", name, index);
                    let _ffmpeg_status = Command::new("yt-dlp")
                        .args(&["-x", "--audio-format", "wav",  &url, "-o", &cloned_mp3])
                        .status()
                        .expect("Failed to execute ffmpeg");

                    // ffmpeg -i kim-1.wav -f segment -segment_time 50 -c copy split_kim/kim-1-%d.wav
                    let split_file_name = format!("{}{}-{}-%d.wav", split_mp3_folder, name, index);
                    let _ffmpeg_status = Command::new("ffmpeg")
                        .args(&["-i", &cloned_mp3, "-f", "segment", "-segment_time", "50", "-c", "copy", &split_file_name])
                        .status()
                        .expect("Failed to execute ffmpeg");
                }
                
                let soundeffect_files = fs::read_dir(split_mp3_folder).unwrap();
                let mut mp3s: HashSet<String> = vec![].into_iter().collect();
                for split_file in soundeffect_files {
                    mp3s.insert(split_file.unwrap().path().display().to_string());
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

    // !transform PITCH/STRETCH/REVERB
    "!transform" => {
            if splitmsg.len() < 2 {
                return Ok(())
            }
            
            let transform_settings = &splitmsg.get(1).unwrap();
            let contents = &splitmsg[2..].join(" ");
            let word_count = &splitmsg[2..].len();
            
            let (pitch, stretch, reverb) = parse_transform_settings(
                transform_settings, *word_count); let pitch = format!("{}", pitch);
            let stretch = format!("{}", stretch);
            println!("{} {} {}", pitch, stretch, reverb);
            
            let _ = tx.send(Event::ElevenLabsRequest(subd_types::ElevenLabsRequest{
                source: Some("begin".to_string()),
                message: contents.to_string(),
                username: msg.user_name.to_string(),
                pitch: Some(pitch),
                stretch: Some(stretch),
                reverb,
                
                ..Default::default()
            }));
            Ok(())
        }
        
        "!stretch" => {
            let stretch = &splitmsg.get(1).unwrap();
            let contents = &splitmsg[2..].join(" ");
            
            let _ = tx.send(Event::ElevenLabsRequest(subd_types::ElevenLabsRequest{
                source: Some("begin".to_string()),

                message: contents.to_string(),
                username: msg.user_name.to_string(),

                stretch: Some(stretch.to_string()),
                ..Default::default()
            }));
            Ok(())
        }
        
        "!pitch" => {
            let pitch = &splitmsg.get(1).unwrap();
            let contents = &splitmsg[2..].join(" ");
            
            let _ = tx.send(Event::ElevenLabsRequest(subd_types::ElevenLabsRequest{
                source: Some("begin".to_string()),
                // What is the message voice_text?
                message: contents.to_string(),
                username: msg.user_name.to_string(),
                pitch: Some(pitch.to_string()),

                ..Default::default()
            }));
            
            Ok(())
        }
        
        "!reverb" => {
            let contents = &splitmsg[1..].join(" ");
            
            let _ = tx.send(Event::ElevenLabsRequest(subd_types::ElevenLabsRequest{
                source: Some("begin".to_string()),
                message: contents.to_string(),
                username: msg.user_name.to_string(),
                reverb: true,
                ..Default::default()
            }));
            
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

        "!dalle" => {
            let prompt = splitmsg.iter().skip(1).map(AsRef::as_ref).collect::<Vec<&str>>().join(" ");
            println!("Dalle Time!");
            let _ = dalle::dalle_time(prompt, msg.user_name).await;
            Ok(())
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

        // !upload_image URL
        // "!upload_image" => handlers::upload_image(msg),

        // ===========================================
        // == Scrolling
        // ===========================================

        // !scroll SOURCE SCROLL_SETTING SPEED DURATION (in milliseconds)
        // !scroll begin x 5 300
        "!scroll" => {
            let default_filter_setting_name = String::from("speed_x");
            
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);
            let filter_setting_name: String = match filter_setting_name.as_str()
            {
                "x" => String::from("speed_x"),
                "y" => String::from("speed_y"),
                _ => default_filter_setting_name,
            };

            // Starting to Scroll: begin speed_x
            println!("Starting to Scroll: {} {}", source, filter_setting_name);

            // TODO: Fix
            // move_transition::update_and_trigger_move_value_filter(
            //     source,
            //     obs::MOVE_SCROLL_FILTER_NAME,
            //     &filter_setting_name,
            //     filter_value,
            //     "",
            //     duration,
            //     2,
            //     &obs_client,
            // )
            // .await
            Ok(())
        }

        // ===========================================
        // == Blur
        // ===========================================
        "!blur" => {
            let _filter_value = splitmsg
                .get(2)
                .map_or(100.0, |x| x.trim().parse().unwrap_or(100.0));

            Ok(())
            // move_transition::update_and_trigger_move_value_filter(
            //     source,
            //     obs::MOVE_BLUR_FILTER_NAME,
            //     "Filter.Blur.Size",
            //     filter_value,
            //     "",
            //     duration,
            //     0,
            //     &obs_client,
            // )
            // .await
        }

        // TODO: Update these values to be variables so we know what they do
        "!noblur" | "!unblur" => {
            Ok(())
            // move_transition::update_and_trigger_move_value_filter(
            //     source,
            //     obs::DEFAULT_BLUR_FILTER_NAME,
            //     "Filter.Blur.Size",
            //     0.0,
            //     5000,
            //     "",
            //     2,
            //     &obs_client,
            // )
            // .await
        }

        // ===========================================
        // == Scaling Sources
        // ===========================================
        "!grow" | "!scale" => {
            let x: f32 = splitmsg
                .get(2)
                .and_then(|temp_x| temp_x.trim().parse().ok())
                .unwrap_or(1.0);
            let y: f32 = splitmsg
                .get(3)
                .and_then(|temp_y| temp_y.trim().parse().ok())
                .unwrap_or(1.0);

            let temp_scene = "Primary";

            println!("\n\tkicking off grow!");
            // This is the real solo use of scale_source
            let res = obs_source::scale_source(
                &temp_scene,
                source,
                x,
                y,
                &obs_client,
            )
            .await;

            if let Err(e) = res {
                let err_msg = format!("Error Scaling {:?}", e);
                send_message(twitch_client, err_msg).await?;
            }
            
            Ok(())
        }

        // ===========================================
        // == Moving Sources
        // ===========================================
        "!move" => {
            let temp_scene = "Primary";
            
            println!("\n!move {} {}", temp_scene, source);

            if splitmsg.len() > 3 {
                let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
                let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);

               let _ = obs_source::move_source(temp_scene, source, x, y, &obs_client).await;
            } else {
                send_message(twitch_client, "Missing X and Y").await?; 
            }

            Ok(())
        }
        
        "!gg" => {
            let x: f32 = splitmsg
                .get(2)
                .and_then(|temp_x| temp_x.trim().parse().ok())
                .unwrap_or(1.0);
            let y: f32 = splitmsg
                .get(3)
                .and_then(|temp_y| temp_y.trim().parse().ok())
                .unwrap_or(1.0);

            let base_scale = Scale {
                x: Some(x),
                y: Some(y),
            };

            let temp_scene = "Primary";
            let res = obs_source::old_trigger_grow(
                &temp_scene,
                source,
                &base_scale,
                x,
                y,
                &obs_client,
            )
            .await;

            if let Err(e) = res {
                let err_msg = format!("Error Scaling {:?}", e);
                send_message(twitch_client, err_msg).await?;
            }
            
            Ok(())
        }

        // TODO: I'd like one-for every corner
        "!tr" => {
            println!("Scene: {} | Source: {}", scene, source);
            move_transition_effects::top_right(&scene, source, &obs_client)
                .await
        }

        "!bl" => {
            move_transition_effects::bottom_right(&scene, source, &obs_client)
                .await
        }

        // ===========================================
        // == Showing/Hiding Sources & Scenes
        // ===========================================
        "!memes" => {
            obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                obs::MEME_SCENE,
                true,
                &obs_client,
            )
            .await
        }

        "!nomemes" | "!nojokes" | "!work" => {
            obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                obs::MEME_SCENE,
                false,
                &obs_client,
            )
            .await
        }

        // Rename These Commands
        "!chat" => obs_hotkeys::trigger_hotkey("OBS_KEY_L", &obs_client).await,

        "!code" => obs_hotkeys::trigger_hotkey("OBS_KEY_H", &obs_client).await,

        "!hide" => obs_source::hide_sources(obs::MEME_SCENE, &obs_client).await,

        "!show" => {
            obs_source::set_enabled(obs::MEME_SCENE, source, true, &obs_client)
                .await
        }
        
        // ===========================================
        // == HotKeys
        // ===========================================
        "!hk" => {
            let key = splitmsg[1].as_str().to_uppercase();
            let obs_formatted_key =  format!("OBS_KEY_{}", key);
            let _ = tx.send(Event::TriggerHotkeyRequest(subd_types::TriggerHotkeyRequest{
                hotkey: obs_formatted_key,
            }));
            Ok(())
        }

        // ===========================================
        // == Creating Scenes & Filters
        // ===========================================
        "!create_source" => {
            let new_scene: obws::requests::scene_items::CreateSceneItem =
                obws::requests::scene_items::CreateSceneItem {
                    scene: obs::DEFAULT_SCENE,
                    source: &source,
                    enabled: Some(true),
                };

            obs_client.scene_items().create(new_scene).await?;
            Ok(())
        }

        "!create_3d_filters" => {
            bootstrap::create_split_3d_transform_filters(source, &obs_client)
                .await
        }

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            bootstrap::create_filters_for_source(source, &obs_client).await
        }

        // ===========================================
        // == Debug Info
        // ===========================================
        "!source" => {
            obs_source::print_source_info(source, &scene, &obs_client).await
        }
        
        "!filter" => {
            
            let default_filter_name  = "Move-3D-Transform-Orthographic".to_string();
            let source: &str = splitmsg.get(1).unwrap_or(&default_filter_name);
            let filter_details =
                match obs_client.filters().get("begin",source).await {
                    Ok(val) => Ok(val),
                    Err(err) => Err(err),
                }?;

            println!("------------------------");
            println!("\n\tFilter ettings: {:?}", filter_details);
            println!("------------------------");
            Ok(())
        }

        // This doesn't seem like it would just be info
        // ...but it is!
        "!outline" => sdf_effects::outline(source, &obs_client).await,

        // ===========================================
        // == Compound Effects
        // ===========================================
        "!norm" => obs_combo::norm(&source, &obs_client).await,

        "!follow" => {
            let scene = obs::DEFAULT_SCENE;
            let leader = splitmsg.get(1).unwrap_or(&default_source);
            let source = leader;

            obs_combo::follow(source, scene, leader, &obs_client).await
        }
        "!staff" => obs_combo::staff(obs::DEFAULT_SOURCE, &obs_client).await,

        // ===============================================================================================
        // ===============================================================================================
        // ===============================================================================================
        // ===============================================================================================

        "!spin" | "!spinx" | "spiny" => {
            // let default_filter_setting_name = String::from("z");
            // let filter_setting_name =
            //     splitmsg.get(2).unwrap_or(&default_filter_setting_name);
            //
            // println!("!spin time {} - {}", source, filter_setting_name);
            // move_transition_effects::spin(
            //     source,
            //     filter_setting_name,
            //     filter_value,
            //     duration,
            //     &obs_client,
            // )
            // .await
            
            let source = "begin";
            let filter_name = "3D-Transform-Perspective";
            
            let new_settings = move_transition::MoveMultipleValuesSetting{
                // filter: Some(filter_name.to_string()),
                // scale_x: Some(217.0),
                // scale_y: Some(200.0),
                rotation_z: Some(500000.0),
                field_of_view: Some(108.0),
                //
                // // If a previous Move_transition set this and you don't reset it, you're gonna hate
                // // you life
                // position_y: Some(0.0),
                ..Default::default()
            };

            let duration = 30000;
            dbg!(&new_settings);
            let three_d_transform_filter_name = filter_name;
            let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
            
            _ = move_transition::update_and_trigger_move_values_filter(
                source,
                &move_transition_filter_name,
                duration,
                new_settings,
                &obs_client,
            )
            .await;
            Ok(())
        }

        // !3d SOURCE FILTER_NAME FILTER_VALUE DURATION
        // !3d begin Rotation.Z 3600 5000
        "!3d" => {
            // If we don't at least have a filter_name, we can't proceed
            if splitmsg.len() < 3 {
                bail!("We don't have a filter name, can't proceed");
            }

            // TODO: This should be done with unwrap
            // What is the default???
            let filter_setting_name = &splitmsg[2];

            move_transition_effects::trigger_3d(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        // ===========================================
        // == Skybox
        // ===========================================
        "!previous" => {
            let default_skybox_id = String::from("2449796");
            let skybox_id: &str = splitmsg.get(1).unwrap_or(&default_skybox_id);
            let file_path =
                "/home/begin/code/BeginGPT/tmp/current/previous.txt";
            if let Err(e) = skybox::write_to_file(file_path, &skybox_id) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to Return to previous Skybox! {}", skybox_id);
            Ok(())
        }

        // This needs to take an ID
        "!styles" => {
            let go_executable_path =
                "/home/begin/code/BeginGPT/GoBeginGPT/bin/GoBeginGPT";
            let styles_flag = "-styles";
            let output = Command::new(go_executable_path)
                .arg(styles_flag)
                .output()
                .expect("Failed to execute Go program.");

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                println!("Output: {}", result);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error: {}", error);
            }
            Ok(())
        }

        // We need to eventually take in style IDs
        "!skybox" => {
            println!("Trying Skybox");
            
            let skybox_info = splitmsg
                .clone()
                .into_iter()
                .skip(1)
                .collect::<Vec<String>>()
                .join(" ");
            
                let _ = tx.send(Event::SkyboxRequest(subd_types::SkyboxRequest{
                    msg: skybox_info,
                }));
        
            // let file_path = "/home/begin/code/BeginGPT/tmp/current/skybox.txt";
            // if let Err(e) = write_to_file(file_path, &skybox_info) {
            //     eprintln!("Error writing to file: {}", e);
            // }
            //
            // println!("Attempting to Generate Skybox! {}", skybox_info);

            Ok(())
        }

        "!remix" => {
            let remix_info = splitmsg
                .clone()
                .into_iter()
                .skip(1)
                .collect::<Vec<String>>()
                .join(" ");
            let file_path = "/home/begin/code/BeginGPT/tmp/current/remix.txt";
            if let Err(e) = skybox::write_to_file(file_path, &remix_info) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to  Remix! {}", remix_info);

            // OK NOw
            // Just save this
            Ok(())
        }

        // ===========================================
        // == Characters
        // ===========================================

        "!talk" => {
            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: "Hello".to_string(),
                    text_source: obs::SOUNDBOARD_TEXT_SOURCE_NAME.to_string(),
                    // text_source: "Soundboard-Text".to_string(),
                },
            ));
            Ok(())
        }
        
        // This Creates a new soundboard text item
        "!soundboard_text" => {
            move_transition_bootstrap::create_soundboard_text(obs_client).await
        }

        "!set_character" => Ok(()),

        "!character" => {
            stream_character::create_new_obs_character(source, obs_client)
                .await?;
            Ok(())
        }

        _ => Ok(()),
    };

    // TODO: Check for a playlist
    let exists = music_scenes::VOICE_TO_MUSIC.iter().any(|&(cmd, _)| cmd == command);
    if exists {
        if !is_mod && !is_vip {
            return Ok(());
        }
        

        let mut scene_details = None;
        for &(cmd, ref scene) in music_scenes::VOICE_TO_MUSIC.iter() {
            if cmd == command {
                scene_details = Some(scene);
                break;
            }
        }

        let mut set_global_voice = true;

        if let Some(details) = scene_details {
            match details.playlist_folder {
                Some(playlist_folder) => {
                    match get_random_mp3_file_name(playlist_folder) {
                        Some(music_filename) => {

                            let items = obs_client.scene_items().list(background_scene).await?;
                            for item in items {
                                let enabled = obs_client.scene_items().enabled(background_scene, item.id).await.unwrap();

                                // println!("Enabled: {}", enabled);
                                // println!("Item: {:?}", item);

                                if enabled && item.source_name == details.music {
                                    println!("We are just changing the music!");
                                    
                                    let _ = obs_source::hide_source(background_scene, details.music, obs_client).await;
                                    set_global_voice = false;
                                }
                            }

                            // BackgroundMusic scene
                            // Now we just need to update the Ffmpeg Source
                            // Now I have to use this model
                            let color_range = obws::requests::custom::source_settings::ColorRange::Auto;
                            
                            let path = Path::new(&music_filename);
                                
                            let media_source = obws::requests::custom::source_settings::FfmpegSource{
                                is_local_file: true,
                                local_file: path,
                                looping: true,
                                restart_on_activate: true,
                                close_when_inactive: true,
                                clear_on_media_end: false,
                                speed_percent: 100,
                                color_range,

                                // Non-Local settings
                                buffering_mb: 1,
                                seekable: false,
                                input: "",
                                input_format: "",
                                reconnect_delay_sec: 1,
                                // ..Default::default()
                            };
                            let set_settings = obws::requests::inputs::SetSettings{
                                settings: &media_source,
                                input: details.music,
                                overlay: Some(true),
                            };
                            let _ = obs_client.inputs().set_settings(set_settings).await;
                        }
                        None => {
                            println!("Could not find a random mp3 file in the playlist folder");
                        }
                    }
                } 
                None => {}
            };
            
            // Hide all Background Music Sources
            let music_list: Vec<&str> = music_scenes::VOICE_TO_MUSIC.iter()
                .map(|(_, scene)| scene.music)
                .collect();
            for source in music_list.iter() {
                let _ = obs_source::hide_source(background_scene, source, obs_client).await;
            }

            // I think we need a gap, to allow the pervious media source update to finish
            let ten_millis = time::Duration::from_millis(300);
            thread::sleep(ten_millis);
            
            // Do
            let _ = obs_source::show_source(background_scene, details.music, obs_client).await;
            // If we have a playlist that isn't None, then we need to first get a RANDOM
            // mp3 from the playlist folder
            //
            // then we update the OBS source w/ the new Media
            // Turn on the Music for the scene
            
            if command == "!sigma" {
                println!("We are in Chad mode!");
                let source = "begin";
                let filter_name = "3D-Transform-Perspective";
                
                let new_settings = move_transition::MoveMultipleValuesSetting{
                    filter: Some(filter_name.to_string()),
                    scale_x: Some(217.0),
                    scale_y: Some(200.0),
                    rotation_x: Some(50.0),
                    field_of_view: Some(108.0),

                    // If a previous Move_transition set this and you don't reset it, you're gonna hate
                    // you life
                    position_y: Some(0.0),
                    ..Default::default()
                };

                dbg!(&new_settings);
                let three_d_transform_filter_name = filter_name;
                let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
                
                _ = move_transition::update_and_trigger_move_values_filter(
                    source,
                    &move_transition_filter_name,
                    duration,
                    new_settings,
                    &obs_client,
                )
                .await;
            }

            // Set the Voice for Begin, which is the source of the global voice
            let _ = uberduck::set_voice(
                details.voice.to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
        
            // Enable Global Voice Mode
            if set_global_voice {
                twitch_stream_state::turn_on_global_voice(&pool)
                    .await?;
            }
        } else {
            println!("Could not find voice info for command.");
        }
    }
    Ok(())
}

fn get_random_mp3_file_name(folder_path: &str) -> Option<String> {
    let full_path = format!("/home/begin/stream/Stream/BackgroundMusic/{}", folder_path);
    let paths = fs::read_dir(full_path).ok()?;

    let mp3_files: Vec<_> = paths
        .filter_map(Result::ok)
        .filter(|dir_entry| {
            dir_entry.path().extension().and_then(|ext| ext.to_str()) == Some("mp3")
        })
        .collect();

    if mp3_files.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    let selected_file = mp3_files.choose(&mut rng).unwrap();

    let new_music = selected_file.file_name().to_str().map(String::from).unwrap();
    let full_path = format!("/home/begin/stream/Stream/BackgroundMusic/{}/{}", folder_path, new_music);
    Some(full_path)
}

async fn from_clone(voice_clone: VoiceClone, api_base_url_v1: &str) -> Result<String, Error> {
    let api_key = env::var("ELEVENLABS_API_KEY").expect("Expected ELEVENLABS_API_KEY in .env");
    let client = Client::new();
    let url = format!("{}/voices/add", api_base_url_v1);

    let mut form = Form::new()
        .text("name", voice_clone.name)
        .text("description", voice_clone.description)
        .text("labels", serde_json::to_string(&voice_clone.labels).unwrap());


    for file_path in voice_clone.files {
        let path = Path::new(&file_path);
        let mut file = File::open(path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        let part = Part::bytes(contents).file_name(file_path).mime_str("audio/mpeg")?;
        form = form.part("files", part);
    }

    let response = client.post(url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        let voice_id: String = response.json::<serde_json::Value>().await?.get("voice_id").unwrap().as_str().unwrap().to_string();
        Ok(voice_id)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}


fn parse_transform_settings(transform_settings: &str, word_count: usize) -> (i32, f32, bool) {
    let mut pitch: i32 = 0; // Default value
    let mut stretch: f32 = 1.0; // Default value
    let mut reverb: bool = false; // Default value

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
            if parsed_stretch >= 0.1 && parsed_stretch <= stretch_limit  {
                stretch = parsed_stretch;
            }
        }
    }

    if settings.len() > 2 {
        reverb = matches!(settings[2].to_lowercase().as_str(), "true" | "t");
    }

    (pitch, stretch, reverb)
}
