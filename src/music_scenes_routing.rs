// use anyhow::Result;
// use crate::music_scenes;
// use crate::obs;
// use crate::obs_scenes;
// use rand::seq::SliceRandom;
// use crate::obs_source;
// use crate::twitch_stream_state;
// use crate::uberduck;
// use crate::move_transition;
// use dotenv::dotenv;
// use obws::Client as OBSClient;
// use reqwest::multipart::{Form, Part};
// use reqwest::{Client, Error};
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::collections::HashSet;
// use std::env;
// use std::fs::File;
// use std::fs;
// use std::io::{Read, Write};
// use std::path::Path;
// use std::process::Command;
// use std::thread;
// use std::time;
// use subd_types::{Event, UserMessage};
// use tokio::sync::broadcast;
//
//
//
// pub async fn music_scenes_commands(
//     tx: &broadcast::Sender<Event>,
//     obs_client: &OBSClient,
//     pool: &sqlx::PgPool,
//     splitmsg: Vec<String>,
//     msg: UserMessage,
// ) -> Result<()> {
//     let default_source = obs::DEFAULT_SOURCE.to_string();
//
//     let is_mod = msg.roles.is_twitch_mod();
//     let is_vip = msg.roles.is_twitch_vip();
//     let background_scene = "BackgroundMusic";
//
//     let source: &str = splitmsg.get(1).unwrap_or(&default_source);
//
//     let duration: u32 = splitmsg
//         .get(4)
//         .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
//
//     let filter_value = splitmsg
//         .get(3)
//         .map_or(0.0, |x| x.trim().parse().unwrap_or(0.0));
//
//     let scene = match obs_scenes::find_scene(source).await {
//         Ok(scene) => scene.to_string(),
//         Err(_) => obs::MEME_SCENE.to_string(),
//     };
//     
//     let command = splitmsg[0].as_str();
//     let not_beginbot = msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
//
//     // TODO: Check for a playlist
//     let exists = music_scenes::VOICE_TO_MUSIC.iter().any(|&(cmd, _)| cmd == command);
//     if !exists {
//         return Ok(());
//     }
//     
//     if !is_mod && !is_vip {
//         return Ok(());
//     }
//     
//
//     let mut scene_details = None;
//     for &(cmd, ref scene) in music_scenes::VOICE_TO_MUSIC.iter() {
//         if cmd == command {
//             scene_details = Some(scene);
//             break;
//         }
//     }
//
//   // if let Some(details) = scene_details {
//   //           match details.playlist_folder {
//   //               Some(playlist_folder) => {
//   //                   match get_random_mp3_file_name(playlist_folder) {
//
//     let mut set_global_voice = true;
//
//
//     
//
//     // if let Some(details) = scene_details {
//     //     match details.playlist_folder {
//     //         Some(playlist_folder) => {
//     //             match get_random_mp3_file_name(playlist_folder)=> {
//     //                 Some(music_filename) => {
//     //                     let items = obs_client.scene_items().list(background_scene).await?;
//     //                     for item in items {
//     //                         let enabled = obs_client.scene_items().enabled(background_scene, item.id).await.unwrap();
//     //
//     //                         if enabled && item.source_name == details.music {
//     //                             println!("We are just changing the music!");
//     //                             
//     //                             let _ = obs_source::hide_source(background_scene, details.music, obs_client).await;
//     //                             set_global_voice = false;
//     //                         }
//     //                     }
//     //
//     //                     // BackgroundMusic scene
//     //                     // Now we just need to update the Ffmpeg Source
//     //                     // Now I have to use this model
//     //                     let color_range = obws::requests::custom::source_settings::ColorRange::Auto;
//     //                     
//     //                     let path = Path::new(&music_filename);
//     //                         
//     //                     let media_source = obws::requests::custom::source_settings::FfmpegSource{
//     //                         is_local_file: true,
//     //                         local_file: path,
//     //                         looping: true,
//     //                         restart_on_activate: true,
//     //                         close_when_inactive: true,
//     //                         clear_on_media_end: false,
//     //                         speed_percent: 100,
//     //                         color_range,
//     //
//     //                         // Non-Local settings
//     //                         buffering_mb: 1,
//     //                         seekable: false,
//     //                         input: "",
//     //                         input_format: "",
//     //                         reconnect_delay_sec: 1,
//     //                         // ..Default::default()
//     //                     };
//     //                     let set_settings = obws::requests::inputs::SetSettings{
//     //                         settings: &media_source,
//     //                         input: details.music,
//     //                         overlay: Some(true),
//     //                     };
//     //                     let _ = obs_client.inputs().set_settings(set_settings).await;
//     //                 }
//     //             }
//     //     
//     //         
//     //             // Hide all Background Music Sources
//     //             let music_list: Vec<&str> = music_scenes::VOICE_TO_MUSIC.iter()
//     //                 .map(|(_, scene)| scene.music)
//     //                 .collect();
//     //             for source in music_list.iter() {
//     //                 let _ = obs_source::hide_source(background_scene, source, obs_client).await;
//     //             }
//     //
//     //             // I think we need a gap, to allow the pervious media source update to finish
//     //             let ten_millis = time::Duration::from_millis(300);
//     //             thread::sleep(ten_millis);
//     //             
//     //             // Do
//     //             let _ = obs_source::show_source(background_scene, details.music, obs_client).await;
//     //             // If we have a playlist that isn't None, then we need to first get a RANDOM
//     //             // mp3 from the playlist folder
//     //             //
//     //             // then we update the OBS source w/ the new Media
//     //             // Turn on the Music for the scene
//     //             
//     //             if command == "!sigma" {
//     //                 println!("We are in Chad mode!");
//     //                 let source = "begin";
//     //                 let filter_name = "3D-Transform-Perspective";
//     //                 
//     //                 let new_settings = move_transition::MoveMultipleValuesSetting{
//     //                     filter: Some(filter_name.to_string()),
//     //                     scale_x: Some(217.0),
//     //                     scale_y: Some(200.0),
//     //                     rotation_x: Some(50.0),
//     //                     field_of_view: Some(108.0),
//     //
//     //                     // If a previous Move_transition set this and you don't reset it, you're gonna hate
//     //                     // you life
//     //                     position_y: Some(0.0),
//     //                     ..Default::default()
//     //                 };
//     //
//     //                 dbg!(&new_settings);
//     //                 let three_d_transform_filter_name = filter_name;
//     //                 let move_transition_filter_name = format!("Move_{}", three_d_transform_filter_name);
//     //                 
//     //                 _ = move_transition::update_and_trigger_move_values_filter(
//     //                     source,
//     //                     &move_transition_filter_name,
//     //                     duration,
//     //                     new_settings,
//     //                     &obs_client,
//     //                 )
//     //                 .await;
//     //             }
//     //
//     //             // Set the Voice for Begin, which is the source of the global voice
//     //             let _ = uberduck::set_voice(
//     //                 details.voice.to_string(),
//     //                 "beginbot".to_string(),
//     //                 pool,
//     //             )
//     //             .await;
//     //
//     //     // Enable Global Voice Mode
//     //     // if set_global_voice {
//     //     //     twitch_stream_state::turn_on_global_voice(&pool)
//     //     //         .await;
//     //     //     Ok(())
//     //     // }
//     //         }
//     //         None => {Ok(())}
//     //     }
//     // };
//     Ok(())
// }
//
// fn get_random_mp3_file_name(folder_path: &str) -> Option<String> {
//     let full_path = format!("/home/begin/stream/Stream/BackgroundMusic/{}", folder_path);
//     let paths = fs::read_dir(full_path).ok()?;
//
//     let mp3_files: Vec<_> = paths
//         .filter_map(Result::ok)
//         .filter(|dir_entry| {
//             dir_entry.path().extension().and_then(|ext| ext.to_str()) == Some("mp3")
//         })
//         .collect();
//
//     if mp3_files.is_empty() {
//         return None;
//     }
//
//     let mut rng = rand::thread_rng();
//     let selected_file = mp3_files.choose(&mut rng).unwrap();
//
//     let new_music = selected_file.file_name().to_str().map(String::from).unwrap();
//     let full_path = format!("/home/begin/stream/Stream/BackgroundMusic/{}/{}", folder_path, new_music);
//     Some(full_path)
// }