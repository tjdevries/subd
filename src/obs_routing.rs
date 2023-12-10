use crate::ai_scenes;
use crate::bootstrap;
use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::move_transition_effects;
use crate::obs;
use crate::obs_combo;
use crate::obs_hotkeys;
use crate::obs_scenes;
use crate::obs_source;
use crate::skybox::check_skybox_status_and_save;
use crate::twitch_rewards;
use rand::Rng;
use std::env;
use std::fs;
use twitch_oauth2::UserToken;
use uuid::Uuid;
// use rand::{seq::SliceRandom, thread_rng};
// use crate::openai;
// use std::env;
use crate::sdf_effects;
use crate::skybox;
use crate::stream_character;
use crate::twitch_stream_state;
use anyhow::{bail, Result};
use obws;
use obws::requests::scene_items::Scale;
use obws::Client as OBSClient;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time;
use subd_twitch::rewards;
use subd_types::{Event, TransformOBSTextRequest, UserMessage};
use tokio::sync::broadcast;
use twitch_api::HelixClient;
use twitch_chat::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
// use openai::{
//     chat::{ ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
//     set_key,
// };

pub async fn handle_obs_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let default_source = obs::DEFAULT_SOURCE.to_string();

    let _is_mod = msg.roles.is_twitch_mod();
    let _is_vip = msg.roles.is_twitch_vip();
    let _background_scene = "BackgroundMusic";
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

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

    // This fails, and we stop
    // let voice = stream_character::get_voice_from_username(pool, &msg.user_name).await?;

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    //       we need to figure a way to look up the defaults per command
    //       because they could be different types

    let command = splitmsg[0].as_str();
    let _ = match command {
        // Iterate through the json file
        // choose a random scene
        // look up the ID in the DB
        //
        // update the price to be 100
        // post about the Sale in the Chat
        // or go to a scene????KJ,,,
        "!flash_sale" => {
            if not_beginbot {
                return Ok(());
            }
            let broadcaster_id = "424038378";

            let file_path = "/home/begin/code/subd/data/AIScenes.json";
            let contents =
                fs::read_to_string(file_path).expect("Can read file");
            let ai_scenes: ai_scenes::AIScenes =
                serde_json::from_str(&contents.clone()).unwrap();

            // This is need to create Reward Manager
            let twitch_user_access_token =
                env::var("TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN").unwrap();
            let reqwest = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let twitch_reward_client: HelixClient<reqwest::Client> =
                HelixClient::new();
            let token = UserToken::from_existing(
                &reqwest,
                twitch_user_access_token.into(),
                None,
                None,
            )
            .await?;
            let reward_manager = rewards::RewardManager::new(
                &twitch_reward_client,
                &token,
                &broadcaster_id,
            );

            // This returns the default cost to every reward to everyone
            // let default_cost: i32 = 300;
            // let ids = twitch_rewards::update_cost_of_all(pool, default_cost)
            //     .await
            //     .unwrap();
            // for id in ids {
            //     let _ =
            //         reward_manager.update_reward(id, default_cost_usize).await;
            // }

            // https://stackoverflow.com/questions/67443847/how-to-generate-random-numbers-in-async-rust
            let random = {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..ai_scenes.scenes.len())
            };
            let random_scene = &ai_scenes.scenes[random];
            let title = &random_scene.reward_title;

            // If we don't have a reward for that Thang
            let reward_res =
                twitch_rewards::find_by_title(&pool, title.to_string())
                    .await
                    .unwrap();

            // // let _ = reward_manager.update_all(4200).await;
            // for scene in ai_scenes.scenes {
            //     scene.id,,
            //     // We need to choose a random
            //     println!("Scene: {:?}", scene);
            // }

            let flash_cost = 100;
            let _ = reward_manager
                .update_reward(reward_res.twitch_id.to_string(), flash_cost)
                .await;

            let update = twitch_rewards::update_cost(
                &pool,
                reward_res.title.to_string(),
                flash_cost as i32,
            )
            .await
            .unwrap();

            println!("Update: {:?}", update);

            // We need a Twitch update

            let msg = format!(
                "Flash Sale! {} - New Low Price! {}",
                reward_res.title, flash_cost
            );
            let _ = send_message(&twitch_client, msg).await;
            // Use this to update
            // reward_res.twitch_id

            // let res = reward_res.unwrap();
            // println!("Found Reward: {}", res.twitch_id);

            // match reward_res {
            //     Ok(res) => {
            //         println!("Found Reward: {}", res.id);
            //     },
            //     Err(e) => {
            //         println!("Error finding Reward: {}", e);
            //     }
            // };

            // let scene_count

            Ok(())
        }

        "da_faq" => {
            // let random_index = rng.gen_range(0..ai_scenes.scenes.len());
            // let random_scene = &ai_scenes.scenes[random_index];
            // let title = &random_scene.reward_title;

            // let mut rng = thread_rng();
            // let reward_res = twitch_rewards::find_by_title(&pool.clone()).await;
            Ok(())
        }

        "!bootstrap_rewards" => {
            if not_beginbot {
                return Ok(());
            }

            // bootstrap::bootstrap_rewards(&obs_client).await
            // let file_path = "/home/begin/code/subd/data/AIScenes.json";
            let file_path = "/home/begin/code/subd/data/AIScenes.json";
            let contents =
                fs::read_to_string(file_path).expect("Can read file");
            let ai_scenes: ai_scenes::AIScenes =
                serde_json::from_str(&contents).unwrap();

            // This is need to create Reward Manager
            let twitch_user_access_token =
                env::var("TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN").unwrap();
            let reqwest = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let twitch_reward_client: HelixClient<reqwest::Client> =
                HelixClient::new();
            let token = UserToken::from_existing(
                &reqwest,
                twitch_user_access_token.into(),
                None,
                None,
            )
            .await?;

            let broadcaster_id = "424038378";
            let reward_manager = rewards::RewardManager::new(
                &twitch_reward_client,
                &token,
                &broadcaster_id,
            );

            for scene in ai_scenes.scenes {
                if scene.voice == "josh" {
                    let cost = scene.cost * 10;
                    let res = reward_manager
                        .create_reward(&scene.reward_title, cost)
                        .await?;

                    let reward_id = res.as_str();
                    let reward_id = Uuid::parse_str(reward_id)?;

                    let _ = twitch_rewards::save_twitch_rewards(
                        &pool.clone(),
                        scene.reward_title,
                        cost,
                        reward_id,
                        true,
                    )
                    .await;
                }
            }

            Ok(())
        }

        //
        // let ai_scenes_map: HashMap<String, &AIScene> = ai_scenes
        //     .scenes
        //     .iter()
        //     .map(|scene| (scene.reward_title.clone(), scene))
        //     .collect();
        "!test" => {
            // let contents = &splitmsg[1..].join(" ");
            // println!("contents: {}", contents);
            // let res = openai::ask_chat_gpt(
            //     "Description the following".to_string(),
            //     contents.to_string(),
            // )
            // .await;
            // let content = res.unwrap().content.unwrap().to_string();
            //
            // let dalle_res = openai::ask_chat_gpt(
            //     "Turn this into a Dalle prompt: ".to_string(),
            //     content,
            // )
            // .await;
            //
            // let nice_res = dalle_res.unwrap().content.unwrap().to_string();
            // println!("\n\tNice Res: {:?}", nice_res);
            Ok(())
        }
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
            )
            .await;
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
            )
            .await;
            Ok(())
        }

        "!nerd" => {
            println!("Nerd TIME!");

            let source = "begin";
            let filter_name = "3D-Transform-Perspective";

            // See the settings aren't correct
            // We need to convert from the settings of the filter
            let new_settings = move_transition::MoveMultipleValuesSetting {
                filter: Some(filter_name.to_string()),
                scale_x: Some(125.3),
                scale_y: Some(140.6),
                position_y: Some(40.0),
                rotation_x: Some(-51.4),
                duration: Some(duration),

                // Added this to test
                field_of_view: Some(90.0),
                ..Default::default()
            };

            let three_d_transform_filter_name = filter_name;
            let move_transition_filter_name =
                format!("Move_{}", three_d_transform_filter_name);

            _ = move_transition::update_and_trigger_move_values_filter(
                source,
                &move_transition_filter_name,
                new_settings,
                &obs_client,
            )
            .await;

            Ok(())
        }

        "!chad" => {
            let source = "begin";
            let filter_name = "3D-Transform-Perspective";

            let new_settings = move_transition::MoveMultipleValuesSetting {
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
            twitch_stream_state::update_implicit_soundeffects(&pool.clone())
                .await?;
            Ok(())
        }
        "!explicit" => {
            twitch_stream_state::update_explicit_soundeffects(&pool.clone())
                .await?;
            Ok(())
        }
        // returns the current state of stream
        "!state" => {
            let state =
                twitch_stream_state::get_twitch_state(&pool.clone()).await?;
            let msg = format!("Twitch State! {:?}", state);
            send_message(twitch_client, msg).await?;
            // send_message(format!("Twitch State! {:?}", state));
            // twitch_stream_state::update_implicit_soundeffects(false, &pool)
            //     .await?;
            Ok(())
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

                let _ = obs_source::move_source(
                    temp_scene,
                    source,
                    x,
                    y,
                    &obs_client,
                )
                .await;
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
            let obs_formatted_key = format!("OBS_KEY_{}", key);
            let _ = tx.send(Event::TriggerHotkeyRequest(
                subd_types::TriggerHotkeyRequest {
                    hotkey: obs_formatted_key,
                },
            ));
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
            let default_filter_name =
                "Move-3D-Transform-Orthographic".to_string();

            let source: &str = splitmsg.get(1).unwrap_or(&default_filter_name);
            let filter_details =
                match obs_client.filters().get("begin", source).await {
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

        // Examples:
        //           !spin 1080 18000 ease-in-and-out cubic
        //
        //
        // !spin SPIN_AMOUNT DURATION EASING-TYPE EASING-FUNCTION
        "!spin" | "!spinx" | "spiny" => {
            let default_duration = 9001;
            let default_easing_type = "ease-in".to_string();
            let default_easing_function = "cubic".to_string();
            let easing_functions = easing_function_match();
            let easing_types = easing_match();
            let duration: u32 = splitmsg
                .get(2)
                .map_or(default_duration, |x| x.trim().parse().unwrap_or(3000));
            let easing_type = splitmsg.get(3).unwrap_or(&default_easing_type);
            let easing_function =
                splitmsg.get(4).unwrap_or(&default_easing_function);
            let easing_function_index =
                &easing_functions[easing_function.as_str()];
            let easing_type_index = &easing_types[easing_type.as_str()];

            let default_spin_amount = 1080.0;
            let spin_amount: f32 =
                splitmsg.get(1).map_or(default_spin_amount, |x| {
                    x.trim().parse().unwrap_or(default_spin_amount)
                });

            let source = "begin";
            let filter_name = "3D-Transform-Perspective";

            let new_settings = move_transition::MoveMultipleValuesSetting {
                // filter: Some(filter_name.to_string()),
                // scale_x: Some(217.0),
                // scale_y: Some(200.0),
                rotation_z: Some(spin_amount),

                easing_function: Some(*easing_function_index),
                easing_type: Some(*easing_type_index),

                // field_of_view: Some(108.0),
                //
                // // If a previous Move_transition set this and you don't reset it, you're gonna hate
                // // you life
                // position_y: Some(0.0),
                duration: Some(duration),
                ..Default::default()
            };

            dbg!(&new_settings);
            let three_d_transform_filter_name = filter_name;
            let move_transition_filter_name =
                format!("Move_{}", three_d_transform_filter_name);

            _ = move_transition::update_and_trigger_move_values_filter(
                source,
                &move_transition_filter_name,
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
        "!skybox_styles" => {
            let styles = skybox::styles_for_chat().await;
            println!("\n\nStyles Time: {:?}", styles);

            // So we think this code isn't returning all chunks
            let chunks = chunk_string(&styles, 500);
            for chunk in chunks {
                println!("Chunk: {}", chunk);
                send_message(twitch_client, chunk).await?;
            }
            Ok(())
        }

        "!check_skybox" => {
            if not_beginbot {
                return Ok(());
            }

            // obs_client
            let _ = check_skybox_status_and_save(9612607).await;
            Ok(())
        }

        // We need to eventually take in style IDs
        "!skybox" => {
            // if not_beginbot {
            //     return Ok(());
            // }
            let style_id = find_style_id(splitmsg.clone());
            println!("\tStyle ID: {}", style_id);

            let skybox_info = if style_id == 1 {
                splitmsg
                    .clone()
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<String>>()
                    .join(" ")
            } else {
                splitmsg
                    .clone()
                    .into_iter()
                    .skip(2)
                    .collect::<Vec<String>>()
                    .join(" ")
            };

            let _ = tx.send(Event::SkyboxRequest(subd_types::SkyboxRequest {
                msg: skybox_info,
                style_id,
            }));

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

    Ok(())
}

pub fn easing_function_match() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("quadratic", 1),
        ("cubic", 2),
        ("quartic", 3),
        ("quintic", 4),
        ("sine", 5),
        ("circular", 6),
        ("exponential", 7),
        ("elastic", 8),
        ("bounce", 9),
        ("back", 10),
    ])
}

pub fn easing_match() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("nothing", 0),
        ("ease-in", 1),
        ("ease-out", 2),
        ("ease-in-and-out", 3),
    ])
}

fn find_style_id(splitmsg: Vec<String>) -> i32 {
    println!("\t Splitmsg: {:?}", splitmsg);
    // TODO: Do a search on Blockade ID for the values
    let range = 1..47;
    let default_style_id = 1;

    match splitmsg.get(1) {
        Some(val) => match val.parse::<i32>() {
            Ok(iv) => {
                if range.contains(&iv) {
                    return iv;
                } else {
                    return default_style_id;
                }
            }
            Err(_) => {
                return default_style_id;
            }
        },
        None => {
            return default_style_id;
        }
    }
}

fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut last_split = 0;
    let mut current_count = 0;

    for (idx, ch) in s.char_indices() {
        current_count += 1;

        // Check if the current character is a space or we reached the end of the string
        // if ch.is_whitespace() || idx == s.len() - 1 {

        if ch.to_string() == "," || idx == s.len() - 1 {
            if current_count >= chunk_size {
                chunks.push(s[last_split..=idx].to_string());

                last_split = idx + 1;
                current_count = 0;
            }
        }
    }

    if last_split < s.len() {
        chunks.push(s[last_split..].to_string());
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_string() {
        let input = "hello, now";
        let strs = chunk_string(input, 4);
        assert_eq!(strs[0], "hello,");
        assert_eq!(strs[1], " now");
        assert_eq!(strs.len(), 2);
    }

    #[tokio::test]
    async fn test_find_style_id() {
        // We want the style_id returned
        let splitmsg: Vec<String> = vec![
            "!skybox".to_string(),
            "3".to_string(),
            "A Cool House".to_string(),
        ];
        let res = find_style_id(splitmsg);
        assert_eq!(res, 3);

        let splitmsg: Vec<String> =
            vec!["!skybox".to_string(), "A Cool House".to_string()];
        let res = find_style_id(splitmsg);
        assert_eq!(res, 1);

        let splitmsg: Vec<String> = vec![
            "!skybox".to_string(),
            "69".to_string(),
            "A Cool House".to_string(),
        ];
        let res = find_style_id(splitmsg);
        assert_eq!(res, 1);
    }
}
