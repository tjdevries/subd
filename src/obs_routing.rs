use anyhow::{bail, Result};
use crate::bootstrap;
use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::move_transition_effects;
use crate::obs;
use crate::obs_combo;
use crate::obs_hotkeys;
use crate::obs_scenes;
use crate::obs_source;
use crate::sdf_effects;
use crate::stream_character;
use crate::stream_fx;
use crate::twitch_stream_state;
use crate::uberduck;
use twitch_chat::send_message;
use obws::Client as OBSClient;
use obws::requests::scene_items::Scale;
use obws;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::process::Command;
use std::thread;
use std::time;
use subd_types::{Event, UserMessage, TransformOBSTextRequest};
use tokio::sync::broadcast;
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Write;

use twitch_irc::{TwitchIRCClient, SecureTCPTransport, login::StaticLoginCredentials};

// This should probably be moved to another file
const ALLOWED_USERS: [&str; 4] = [
    "beginbot",
    "zanuss",
    "ArtMattDank",
    "carlvandergeest",
];

// We need these to be a map instead
const SOURCES: [&str; 7] = [
    "Yoga-BG-Music",
    "KenBurns-BG-Music",
    "Hospital-BG-Music",
    "Dramatic-BG-Music",
    "Romcom-BG-Music",
    "Sigma-BG-Music",
    "News-1-BG-Music",
];

#[derive(Serialize, Deserialize, Debug)]
struct ImageResponse {
    created: i64,  // Assuming 'created' is a Unix timestamp
    data: Vec<ImageData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    url: String,
    // You can add more fields if your objects have more data
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
    println!("WE GOT A MOD!: {}", is_mod);
    println!("WE GOT A VIP!: {}", is_vip);
    // msg.roles

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
    
   let voice = stream_character::get_voice_from_username(pool, &msg.user_name).await?;

    // So we should look up voice here

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    //       we need to figure a way to look up the defaults per command
    //       because they could be different types

    match splitmsg[0].as_str() {
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

        // ===========================================
        // == Voices
        // ===========================================

        "!nothing" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            // Disable Global Voice Mode
            twitch_stream_state::turn_off_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        "!news" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "News-1-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;

            // Set beginbot to the best Hospital Voices
            let _ = uberduck::set_voice(
                "james".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            // Enable Global Voice Mode
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        
        "!sigma" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "Sigma-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;

            let _ = uberduck::set_voice(
                "ethan".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            // Enable Global Voice Mode
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        "!romcom" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }

            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "Romcom-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;

            // Set beginbot to the best Hospital Voices
            let _ = uberduck::set_voice(
                "fin".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            // Enable Global Voice Mode
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        "!yoga" => {
            // This is the best way
            if !is_mod && !is_vip {
                return Ok(());
            }

            // if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
            //     return Ok(());
            // }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "Yoga-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;

            let _ = uberduck::set_voice(
                "thomas".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        "!dramatic" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "Dramatic-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;
            
            let _ = uberduck::set_voice(
                "ethan".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }


        "!ken" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }
            
            let source = "KenBurns-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;
            
            let _ = uberduck::set_voice(
                "james".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;

            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }

        "!hospital" => {
            if !ALLOWED_USERS.contains(&msg.user_name.as_str()) {
                return Ok(());
            }
            
            let scene = "BackgroundMusic";
            
            for source in SOURCES.iter() {
                let _ = obs_source::hide_source(scene, source, obs_client).await;
            }

            let source = "Hospital-BG-Music";
            let _ = obs_source::show_source(scene, source, obs_client).await;
            
            let _ = uberduck::set_voice(
                "bella".to_string(),
                "beginbot".to_string(),
                pool,
            )
            .await;
            
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        "!random" => {
            Ok(())
            // uberduck::use_random_voice(msg.contents.clone(), msg.user_name, tx)
            //     .await
        }
        
        "!my_voice" | "!myvoice" | "!my_name" | "!myname" => {
            // let voice = msg.voice.to_string();
           let info = format!("{} - {}", msg.user_name, voice);
           send_message(twitch_client, info).await?;
            Ok(())
        }

        // !global_voice Ethan
        "!no_global_voice" => {
            if msg.user_name != "beginbot"  {
                return Ok(())
            }
            
            twitch_stream_state::turn_off_global_voice(&pool)
                .await?;
            Ok(())
        }
        
        // TODO: improve this 
        "!global_voice" => {
            if msg.user_name != "beginbot"  {
                return Ok(())
            }
            
            let default_voice = obs::TWITCH_DEFAULT_VOICE.to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);

            println!("Turning on Global Voice");
            twitch_stream_state::turn_on_global_voice(&pool)
                .await?;
            
            // This should write to somewhere in the DB, that tracks global voices
            // uberduck::set_voice(
            //     voice.to_string(),
            //     msg.user_name.to_string(),
            //     pool,
            // )
            // .await
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
            // how do we read in the message to pass to DAlle
            println!("Dalle Time!");
            dalle_time(msg.contents).await;
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

            move_transition::update_and_trigger_move_value_filter(
                source,
                obs::MOVE_SCROLL_FILTER_NAME,
                &filter_setting_name,
                filter_value,
                duration,
                2,
                &obs_client,
            )
            .await
        }

        // ===========================================
        // == Blur
        // ===========================================
        "!blur" => {
            let filter_value = splitmsg
                .get(2)
                .map_or(100.0, |x| x.trim().parse().unwrap_or(100.0));

            move_transition::update_and_trigger_move_value_filter(
                source,
                obs::MOVE_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                filter_value,
                duration,
                0,
                &obs_client,
            )
            .await
        }

        // TODO: Update these values to be variables so we know what they do
        "!noblur" | "!unblur" => {
            move_transition::update_and_trigger_move_value_filter(
                source,
                obs::DEFAULT_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                0.0,
                5000,
                2,
                &obs_client,
            )
            .await
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

        // Perspective
        // Corner Pin
        // Orthographic
        "!spin" | "!spinx" | "spiny" => {
            let default_filter_setting_name = String::from("z");
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);

            move_transition_effects::spin(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!def_ortho" => {
            stream_fx::default_ortho(source, duration, &obs_client).await
        }
        "!ortho" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            // TODO: This should be done with unwrap
            // What is the default???
            let filter_setting_name = &splitmsg[2];

            stream_fx::trigger_ortho(
                source,
                "3D_Orthographic",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!perp" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            // TODO: This should be done with unwrap
            // What is the default???
            let filter_setting_name = &splitmsg[2];

            stream_fx::trigger_ortho(
                source,
                "3D_Perspective",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!corner" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            // TODO: This should be done with unwrap
            // What is the default???
            let filter_setting_name = &splitmsg[2];

            stream_fx::trigger_ortho(
                source,
                "3D_CornerPin",
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
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
            if let Err(e) = write_to_file(file_path, &skybox_id) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to Return to previous Skybox! {}", skybox_id);
            Ok(())
        }

        // This needs to take an ID
        "!styles" => {
            // HELLO
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
        // One is the name of an OBS Filter
        // One is the name of a custom websocket message
        "!bar1" => {
            _ = trigger_obs_move_filter_and_websocket(
                &obs_client,
                "BeginBar1",
                "bar1",
            )
            .await;
            Ok(())
        }

        // Examples:
        //           !goto OBS_POSITION OPTIONAL_SKYBOX_ID
        //           !goto bar1
        //           !goto bar1 2451596
        //
        //           We save bar1 (which is a set of coordinates for the pannellum)
        "!goto" => {
            let default_skybox_scene = String::from("office");
            let skybox_scene: &str =
                splitmsg.get(1).unwrap_or(&default_skybox_scene);

            let default_skybox_id = String::from("");
            let skybox_id: &str = splitmsg.get(2).unwrap_or(&default_skybox_id);

            println!("Triggering Scene: {} {}", skybox_scene, skybox_id);
            // we need to look up if a file exists
            _ = trigger_scene(&obs_client, &skybox_scene, skybox_id).await;
            Ok(())
        }

        "!bar" => {
            _ = trigger_obs_move_filter_and_websocket(
                &obs_client,
                "BeginBar2",
                "bar",
            )
            .await;
            Ok(())
        }

        "!lunch" => {
            _ = trigger_obs_move_filter_and_websocket(
                &obs_client,
                "BeginOffice2",
                "lunch",
            )
            .await;
            Ok(())
        }

        "!office" => {
            _ = trigger_obs_move_filter_and_websocket(
                &obs_client,
                "BeginOffice1",
                "office",
            )
            .await;
            Ok(())
        }

        "!duet" => {
            // let prompt = splitmsg
            //     .clone()
            //     .into_iter()
            //     .skip(1)
            //     .collect::<Vec<String>>()
            //     .join(" ");

            // We need to use the "duet primer"
            //
            // we need to save in /tmp/current/duet.txt
            // I need to pass this prompt to chatgpt4
            Ok(())
        }

        // We need to eventually take in style IDs
        "!skybox" => {
            
            // So this is the skybox command
            // 
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
            if let Err(e) = write_to_file(file_path, &remix_info) {
                eprintln!("Error writing to file: {}", e);
            }

            println!("Attempting to  Remix! {}", remix_info);

            // OK NOw
            // Just save this
            Ok(())
        }

        // !remix REMIX_ID STYLE_ID "WRITE YOUR PROMPT HERE"
        "!old_remix" => {
            println!("Running Skybox Remix for {}", msg.user_name);

            let go_executable_path =
                "/home/begin/code/BeginGPT/GoBeginGPT/bin/GoBeginGPT";

            let default_remix_id = "2295844".to_string();
            let default_prompt = "danker".to_string();

            let remix_flag = "-remix";
            let remix_id_flag = "-remix_id";
            let prompt_flag = "-prompt";
            let style_flag = "-style";

            // How can I check if the "2" argument is a number
            let remix_id: &str = splitmsg.get(1).unwrap_or(&default_remix_id);

            let style_id_str: &str = splitmsg.get(2).unwrap_or(&default_prompt);
            let style_id_int: Result<i32, _> = style_id_str.parse();

            let style_id = match style_id_int {
                Ok(id) => id,
                Err(_) => 0,
            };

            let prompt_skip = if style_id == 0 { 2 } else { 3 };
            let prompt = splitmsg
                .clone()
                .into_iter()
                .skip(prompt_skip)
                .collect::<Vec<String>>()
                .join(" ");

            // ./bin/GoBeginGPT -remix -remix_id=2295844 -style=20 -prompt="Office covered in Dank Weed"
            let output = Command::new(go_executable_path)
                .arg(remix_flag)
                .arg(remix_id_flag)
                .arg(remix_id)
                .arg(style_flag)
                .arg(style_id_str.clone())
                .arg(prompt_flag)
                .arg(prompt)
                .output()
                .expect("Failed to execute Go program.");

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                println!("Output: {}", result);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error: {}", error);
            }

            // This
            // TODO: Extract out into function
            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                "skybox",
                false,
                &obs_client,
            )
            .await;
            let ten_millis = time::Duration::from_millis(300);
            thread::sleep(ten_millis);
            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                "skybox",
                true,
                &obs_client,
            )
            .await;
            Ok(())
        }

        "!old_skybox" => {
            println!("Running Skybox for {}", msg.user_name);

            let content = msg.contents;
            let file_path = "/home/begin/code/BeginGPT/tmp/user_skybox.txt";
            if let Err(e) = write_to_file(file_path, &content) {
                eprintln!("Error writing to file: {}", e);
            }

            let go_executable_path =
                "/home/begin/code/BeginGPT/GoBeginGPT/bin/GoBeginGPT";
            let argument_prompt_file = "-prompt_file";
            let prompt_file_path = file_path;

            // how do we direct standard out
            let output = Command::new(go_executable_path)
                .arg(argument_prompt_file)
                .arg(prompt_file_path)
                .output()
                .expect("Failed to execute Go program.");

            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                println!("Output: {}", result);
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error: {}", error);
            }

            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                "skybox",
                false,
                &obs_client,
            )
            .await;
            let ten_millis = time::Duration::from_millis(300);
            thread::sleep(ten_millis);
            let _ = obs_source::set_enabled(
                obs::DEFAULT_SCENE,
                "skybox",
                true,
                &obs_client,
            )
            .await;
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

        // ===================================================================

        _ => Ok(()),
    }
}

fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// OBS_filter_name
// skybox_id
pub async fn trigger_scene(
    obs_client: &OBSClient,
    filter_name: &str,
    skybox_id: &str,
) -> Result<()> {
    let scene = "Primary";
    // TODO: make this dynamic
    // let content = "lunch";

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    let file_path = "/home/begin/code/BeginGPT/tmp/current/move.txt";

    let skybox_id_map = HashMap::from([
        ("office".to_string(), "2443168".to_string()),
        ("office1".to_string(), "2443168".to_lowercase()),
        ("bar1".to_string(), "2451051".to_string()),
        ("bar".to_string(), "2449796".to_string()),
    ]);

    // const pannellumMoveFunctions = {
    //   'office1': {
    //     'func': office,
    //     'id': "2443168",
    //     'url': "https://blockade-platform-production.s3.amazonaws.com/images/imagine/skybox_sterile_office_environment_amazon_hq_style__a8185e0b9204af34__2443168_a8185e0b9204af34__.jpg?ver=1",
    //   },
    //   'office': {
    //     'func': office,
    //     'id': "2443168",
    //     'url': "https://blockade-platform-production.s3.amazonaws.com/images/imagine/skybox_sterile_office_environment_amazon_hq_style__a8185e0b9204af34__2443168_a8185e0b9204af34__.jpg?ver=1",
    //   },
    //   'bar':  {
    //     'func': bar,
    //     'id': "2449796",
    //     'url': "https://blockade-platform-production.s3.amazonaws.com/images/imagine/cocktail_bar__b65346d0a00befc9__2449796_b65346d0a00befc9__2449796.jpg?ver=1",
    //   },
    //   'bar1': {
    //     'func': bar1,
    //     'id': "2451051",
    //     'url': "https://blockade-platform-production.s3.amazonaws.com/images/imagine/dirty_dingy_bar_biker_dudes_sticky_floors__b5373f30090673cf__2451051_b.jpg?ver=1",
    //   }
    // };

    // let skybox_path = "";

    let skybox_path = if skybox_id == "" {
        let new_skybox_id = &skybox_id_map[filter_name.clone()];
        format!(
            "/home/begin/code/BeginGPT/GoBeginGPT/skybox_archive/{}.txt",
            new_skybox_id
        )
    } else {
        format!(
            "/home/begin/code/BeginGPT/GoBeginGPT/skybox_archive/{}.txt",
            skybox_id
        )
    };

    // This URL is rare
    // unless you look up ID based on
    println!("Checking for Archive: {}", skybox_path);
    let skybox_url_exists = std::path::Path::new(&skybox_path).exists();

    if skybox_url_exists {
        let url = fs::read_to_string(skybox_path).expect("Can read file");
        let new_skybox_command = format!("{} {}", &filter_name, url);
        if let Err(e) = write_to_file(file_path, &new_skybox_command) {
            eprintln!("Error writing to file: {}", e);
        }
    } else {
        if let Err(e) = write_to_file(file_path, &filter_name) {
            eprintln!("Error writing to file: {}", e);
        }
    }

    // let mut owned_string: String = "hello ".to_owned();
    // let borrowed_string: &str = "world";
    // owned_string.push_str(borrowed_string);
    // println!("{}", owned_string);

    return Ok(());
}

pub async fn trigger_obs_move_filter_and_websocket(
    obs_client: &OBSClient,
    filter_name: &str,
    content: &str,
) -> Result<()> {
    let scene = "Primary";
    // TODO: make this dynamic
    // let content = "lunch";

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    let file_path = "/home/begin/code/BeginGPT/tmp/current/move.txt";

    if let Err(e) = write_to_file(file_path, &content) {
        eprintln!("Error writing to file: {}", e);
    }
    return Ok(());
}

// "!bar" => {
//     _ = trigger_obs_move_filter_and_websocket(
//         &obs_client,
//         "BeginBar2",
//         "bar",
//     )
//     .await;
//     Ok(())
// }

// "!lunch" => {
//     _ = trigger_obs_move_filter_and_websocket(
//         &obs_client,
//         "BeginOffice2",
//         "lunch",
//     )
//     .await;
//     Ok(())
// }

// "!office" => {
//     _ = trigger_obs_move_filter_and_websocket(
//         &obs_client,
//         "BeginOffice1",
//         "office",
//     )
//     .await;
//     Ok(())
// }


async fn dalle_time(contents: String) -> Result<(), reqwest::Error> {
    let api_key = env::var("OPENAI_API_KEY").unwrap();

    // TODO: This is for saving to the file
    // which we aren't doing yet
    let truncated_prompt = contents.chars().take(80).collect::<String>();
    let client = reqwest::Client::new();

    // let size = "1792x1024";
    // let other_size = "1024x1792";
    
    // Not sure 
    // TODO: Update these
    let response = client
        .post("https://api.openai.com/v1/images/generations")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "prompt": contents,
            "n": 4,
            // "size": size,
            // "size": "1080x1080",
            // "size": "1792x1024",
            "size": "1024x1024",
        }))
        .send()
        .await?;
    
    let text = response.text().await?;

    let image_response: Result<ImageResponse, _> = serde_json::from_str(&text);

    match image_response {
        Ok(response) => {
            for (index, image_data) in response.data.iter().enumerate() {
                
                // let filename = format!("{}-{}.png", truncated_prompt, index);
                
                // We should be using the prompt here
                // but then we have to update to saved file
                // we could also just save it twice.
                // One for Archive purposes
                let filename = format!("./tmp/dalle-{}.png", index+1);
                println!("Image URL: {} | ", image_data.url.clone());
                let image_data = reqwest::get(image_data.url.clone()).await?.bytes().await?.to_vec();
                
                let mut file = File::create(filename).unwrap();
                file.write_all(&image_data).unwrap();
                

                // Can we hide and show the Dalle-Gen-1
            }
        },
        Err(e) => {
            eprintln!("Error deserializing response: {}", e);
        }
    }
    
    Ok(())
} 


