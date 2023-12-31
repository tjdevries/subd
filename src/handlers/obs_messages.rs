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
use crate::openai;
use crate::sdf_effects;
use crate::stream_character;
use crate::twitch_rewards;
use crate::twitch_stream_state;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws;
use obws::Client as OBSClient;
use rand::Rng;
use rodio::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::thread;
use std::time;
use subd_twitch::rewards;
use subd_types::{Event, TransformOBSTextRequest, UserMessage};
use tokio::sync::broadcast;
use twitch_api::HelixClient;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_oauth2::UserToken;
use uuid::Uuid;

const PRIMARY_CAM_SCENE: &str = "Begin";
const _DEFAULT_DURATION: u32 = 9001;

pub enum WideArgPosition {
    Source(String),
    X(f32),
    Duration(u64),
}

// pub enum WideRequestPosition {
//     Source(String),
//     X(f32),
//     Duration(u64),
// }

pub enum ChatArgPosition {
    Source(String),
    X(f32),
    Y(f32),
    RotationZ(f32),
    Duration(u64),
    EasingType(String),
    EasingFunction(String),
}

#[derive(Default, Debug)]
pub struct WideRequest {
    source: String,
    scene: String,
    x: f32,
    duration: u64,
}

#[derive(Default, Debug)]
pub struct ChatMoveSourceRequest {
    source: String,
    scene: String,
    x: f32,
    y: f32,
    rotation_z: f32,
    duration: u64,
    easing_type: String,
    easing_function: String,
    easing_type_index: i32,
    easing_function_index: i32,
}

pub struct OBSMessageHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for OBSMessageHandler {
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

            match handle_obs_commands(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &self.sink,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}

pub async fn handle_obs_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<(), String> {
    let default_source = obs::DEFAULT_SOURCE.to_string();
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));

    let scene = match obs_scenes::find_scene(source).await {
        Ok(scene) => scene.to_string(),
        Err(_) => obs::MEME_SCENE.to_string(),
    };

    let command = splitmsg[0].as_str();

    let _ = match command {
        "!flash_sale" => {
            if not_beginbot {
                return Ok(());
            }
            let res = flash_sale(pool, twitch_client).await;
            return res;
        }

        "!bootstrap_rewards" => {
            if not_beginbot {
                return Ok(());
            }

            let file_path = "/home/begin/code/subd/data/AIScenes.json";
            let contents =
                fs::read_to_string(file_path).expect("Can read file");
            let ai_scenes: ai_scenes::AIScenes =
                serde_json::from_str(&contents).map_err(|e| e.to_string())?;

            // This is need to create Reward Manager
            let twitch_user_access_token =
                env::var("TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN").unwrap();
            let reqwest = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| e.to_string())?;
            let twitch_reward_client: HelixClient<reqwest::Client> =
                HelixClient::new();
            let token = UserToken::from_existing(
                &reqwest,
                twitch_user_access_token.into(),
                None,
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

            let broadcaster_id = "424038378";
            let reward_manager = rewards::RewardManager::new(
                &twitch_reward_client,
                &token,
                &broadcaster_id,
            );

            // WE could make this more dynamic
            for scene in ai_scenes.scenes {
                if scene.reward_title == "Comedy Trailer" {
                    let cost = scene.cost * 10;
                    let res = reward_manager
                        .create_reward(&scene.reward_title, cost)
                        .await
                        .map_err(|e| e.to_string())?;

                    let reward_id = res.as_str();
                    let reward_id = Uuid::parse_str(reward_id)
                        .map_err(|e| e.to_string())?;

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

            return Ok(());
        }

        // =================== //
        // === Experiments === //
        // =================== //
        // !wide SOURCE WIDTH DURATION
        "!wide" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = vec![
                WideArgPosition::Source("beginbot".to_string()),
                WideArgPosition::X(500.0),
                WideArgPosition::Duration(3000),
            ];
            let req = build_wide_request(meat_of_message, arg_positions)?;
            let filter_value = 300.0;
            let filter_name = "3D-Transform-Orthographic";
            let filter_setting_name = "Scale.X";
            let _ = move_transition_effects::trigger_move_value_3d_transform(
                &req.source,
                filter_name,
                filter_setting_name,
                filter_value,
                req.duration as u32,
                obs_client,
            )
            .await;

            return Ok(());
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
            obs_client
                .filters()
                .set_enabled(filter_enabled)
                .await
                .map_err(|e| e.to_string())?;

            let filter_name = "Default_3D-Transform-Orthographic";
            let filter_enabled = obws::requests::filters::SetEnabled {
                source: "begin",
                filter: &filter_name,
                enabled: true,
            };
            obs_client
                .filters()
                .set_enabled(filter_enabled)
                .await
                .map_err(|e| e.to_string())?;

            let filter_name = "Default_3D-Transform-CornerPin";
            let filter_enabled = obws::requests::filters::SetEnabled {
                source: "begin",
                filter: &filter_name,
                enabled: true,
            };
            obs_client
                .filters()
                .set_enabled(filter_enabled)
                .await
                .map_err(|e| e.to_string())?;

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
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        "!explicit" => {
            twitch_stream_state::update_explicit_soundeffects(&pool.clone())
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        // returns the current state of stream
        "!state" => {
            let state = twitch_stream_state::get_twitch_state(&pool.clone())
                .await
                .map_err(|e| e.to_string())?;
            let msg = format!("Twitch State! {:?}", state);
            send_message(twitch_client, msg)
                .await
                .map_err(|e| e.to_string())?;
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
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = vec![
                ChatArgPosition::Source("beginbot".to_string()),
                ChatArgPosition::X(500.0),
                ChatArgPosition::Y(500.0),
                ChatArgPosition::Duration(3000),
                ChatArgPosition::EasingType("ease-in".to_string()),
                ChatArgPosition::EasingFunction("bounce".to_string()),
            ];
            let req =
                build_chat_move_source_request(meat_of_message, arg_positions);

            dbg!(&req);

            move_transition_effects::scale_source(
                &req.scene,
                &req.source,
                req.x,
                req.y,
                req.duration as u64,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await
        }

        // ===========================================
        // == Moving Sources
        // ===========================================

        // TODO: I'd like one-for every corner
        "!tr" => {
            move_transition_effects::top_right(
                &PRIMARY_CAM_SCENE,
                source,
                &obs_client,
            )
            .await
        }

        "!br" => {
            move_transition_effects::bottom_right(
                &PRIMARY_CAM_SCENE,
                source,
                &obs_client,
            )
            .await
        }

        "!alex" => {
            let source = "alex";
            let scene = "memes";

            let arg_positions = vec![
                ChatArgPosition::X(1111.0),
                ChatArgPosition::Y(500.0),
                ChatArgPosition::Duration(3000),
                ChatArgPosition::EasingType("ease-in".to_string()),
                ChatArgPosition::EasingFunction("bounce".to_string()),
            ];
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            move_transition_effects::move_source_in_scene_x_and_y(
                scene,
                source,
                req.x,
                req.y,
                req.duration,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await
        }

        "!begin" => {
            let source = "begin";
            let scene = PRIMARY_CAM_SCENE;

            let arg_positions = vec![
                ChatArgPosition::X(1111.0),
                ChatArgPosition::Y(500.0),
                ChatArgPosition::Duration(3000),
                ChatArgPosition::EasingType("ease-in".to_string()),
                ChatArgPosition::EasingFunction("bounce".to_string()),
            ];
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            move_transition_effects::move_source_in_scene_x_and_y(
                scene,
                source,
                req.x,
                req.y,
                req.duration,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await
        }

        // So we save a source
        // Lets add a timestamp
        "!save" => {
            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let unique_identifier = format!("{}_screenshot.png", timestamp);
            let filename = format!(
                "/home/begin/code/subd/tmp/screenshots/{}",
                unique_identifier
            );
            if let Err(e) =
                obs_source::save_screenshot(&obs_client, "begin", &filename)
                    .await
            {
                eprintln!("Error Saving Screenshot: {}", e);
                return Ok(());
            };

            // WHAT
            if let Ok(res) = openai::ask_gpt_vision2(&filename, None).await {
                dbg!(&res);
            }

            Ok(())
        }

        // !move MEME_NAME X Y DURATION EASE-TYPE EASE-FUNCTION
        "!move" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = vec![
                ChatArgPosition::Source("beginbot".to_string()),
                ChatArgPosition::X(500.0),
                ChatArgPosition::Y(500.0),
                ChatArgPosition::Duration(3000),
                ChatArgPosition::EasingType("ease-in".to_string()),
                ChatArgPosition::EasingFunction("cubic".to_string()),
            ];
            let req =
                build_chat_move_source_request(meat_of_message, arg_positions);

            move_transition_effects::move_source_in_scene_x_and_y(
                &req.scene,
                &req.source,
                req.x,
                req.y,
                req.duration as u64,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
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

            obs_client
                .scene_items()
                .create(new_scene)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        "!create_3d_filters" => {
            Ok(())
            // bootstrap::create_split_3d_transform_filters(source, &obs_client)
            //     .await
        }

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            if not_beginbot {
                return Ok(());
            }
            let default = "alex".to_string();
            let source: &str = splitmsg.get(1).unwrap_or(&default);
            _ = bootstrap::remove_all_filters(source, &obs_client).await;
            bootstrap::create_split_3d_transform_filters(source, &obs_client)
                .await
        }

        // ===========================================
        // == Debug Info
        // ===========================================
        "!source" => {
            obs_source::print_source_info(source, &scene, &obs_client).await
        }

        "!filter" => {
            let default_filter_name = "Move_begin".to_string();
            // "Move-3D-Transform-Orthographic".to_string();

            let source: &str = splitmsg.get(1).unwrap_or(&default_filter_name);
            let filter_details =
                match obs_client.filters().get("Begin", source).await {
                    Ok(val) => Ok(val),
                    Err(err) => Err(err),
                }
                .map_err(|e| e.to_string())?;

            println!("------------------------");
            println!("\n\tFilter Settings: {:?}", filter_details);
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
            let arg_positions = vec![
                ChatArgPosition::Source("beginbot".to_string()),
                ChatArgPosition::RotationZ(1080.0),
                ChatArgPosition::Duration(3000),
                ChatArgPosition::EasingType("ease-in-and-out".to_string()),
                ChatArgPosition::EasingFunction("sine".to_string()),
            ];
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            dbg!(&req);

            move_transition_effects::spin_source(
                &req.source,
                req.rotation_z,
                req.duration,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await
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
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        _ => Ok(()),
    };

    Ok(())
}

pub fn build_chat_move_source_request(
    splitmsg: Vec<String>,
    arg_positions: Vec<ChatArgPosition>,
) -> ChatMoveSourceRequest {
    let default_source = "begin".to_string();
    let default_scene = PRIMARY_CAM_SCENE.to_string();

    let mut req = ChatMoveSourceRequest {
        ..Default::default()
    };

    for (index, arg) in arg_positions.iter().enumerate() {
        match arg {
            ChatArgPosition::Source(source) => {
                req.source = splitmsg.get(index).unwrap_or(source).to_string();
            }
            ChatArgPosition::RotationZ(z) => {
                let str_z = format!("{}", z);
                req.rotation_z =
                    splitmsg.get(index).unwrap_or(&str_z).parse().unwrap_or(*z);
            }
            ChatArgPosition::X(x) => {
                let str_x = format!("{}", x);
                req.x =
                    splitmsg.get(index).unwrap_or(&str_x).parse().unwrap_or(*x);
            }
            ChatArgPosition::Y(y) => {
                let str_y = format!("{}", y);
                req.y = splitmsg
                    .get(index)
                    .unwrap_or(&str_y)
                    .to_string()
                    .parse()
                    .unwrap_or(*y);
            }
            ChatArgPosition::Duration(duration) => {
                let str_duration = format!("{}", duration);
                req.duration = splitmsg
                    .get(index)
                    .unwrap_or(&str_duration)
                    .to_string()
                    .parse()
                    .unwrap_or(*duration);
            }
            ChatArgPosition::EasingType(easing_type) => {
                req.easing_type =
                    splitmsg.get(index).unwrap_or(easing_type).to_string()
            }
            ChatArgPosition::EasingFunction(easing_function) => {
                req.easing_function =
                    splitmsg.get(index).unwrap_or(easing_function).to_string()
            }
        }
    }

    let (easing_type_index, easing_function_index) = find_easing_indicies(
        req.easing_type.clone(),
        req.easing_function.clone(),
    );

    req.easing_type_index = easing_type_index;
    req.easing_function_index = easing_function_index;

    let new_begin_source = obs::NEW_BEGIN_SOURCE;
    let scene = if req.source == "begin" {
        default_scene
    } else if req.source == new_begin_source {
        "AIAssets".to_string()
    } else {
        "Memes".to_string()
    };

    req.scene = scene;

    return req;
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

pub fn chunk_string(s: &str, chunk_size: usize) -> Vec<String> {
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

fn find_easing_indicies(
    easing_type: String,
    easing_function: String,
) -> (i32, i32) {
    let easing_types = easing_match();
    let easing_functions = easing_function_match();
    let easing_type_index =
        easing_types.get(easing_type.clone().as_str()).unwrap_or(&1);
    let easing_function_index = easing_functions
        .get(easing_function.clone().as_str())
        .unwrap_or(&1);

    (*easing_type_index, *easing_function_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_screenshotting() {
        let obs_client = obs::create_obs_client().await.unwrap();

        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let unique_identifier = format!("{}_screenshot.png", timestamp);
        let filename = format!(
            "/home/begin/code/subd/tmp/screenshots/fake/{}",
            unique_identifier
        );
        let _ = obs_source::save_screenshot(&obs_client, "Primary", &filename)
            .await;
    }

    #[test]
    fn test_chunk_string() {
        let input = "hello, now";
        let strs = chunk_string(input, 4);
        assert_eq!(strs[0], "hello,");
        assert_eq!(strs[1], " now");
        assert_eq!(strs.len(), 2);
    }

    // Now we can test
    #[test]
    fn test_easing_index() {
        let res =
            find_easing_indicies("ease-in".to_string(), "bounce".to_string());
        assert_eq!(res, (1, 9));
    }
}

pub fn build_wide_request(
    splitmsg: Vec<String>,
    arg_positions: Vec<WideArgPosition>,
) -> Result<WideRequest, String> {
    let default_source = "begin".to_string();
    let default_scene = PRIMARY_CAM_SCENE.to_string();

    let mut req = WideRequest {
        ..Default::default()
    };

    for (index, arg) in arg_positions.iter().enumerate() {
        match arg {
            WideArgPosition::Source(source) => {
                req.source = splitmsg.get(index).unwrap_or(source).to_string()
            }
            WideArgPosition::X(x) => {
                if let Some(x) = splitmsg
                    .get(index)
                    .and_then(|m| Some(m.parse::<f32>().unwrap_or(100.0)))
                {
                    req.x = x
                }
            }
            WideArgPosition::Duration(duration) => {
                if let Some(duration) = splitmsg
                    .get(index)
                    .and_then(|m| Some(m.parse::<u64>().unwrap_or(3000)))
                {
                    req.duration = duration
                }
            }
        }
    }

    return Ok(req);
}

pub async fn flash_sale(
    pool: &sqlx::PgPool,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
) -> Result<(), String> {
    let broadcaster_id = "424038378";
    let file_path = "/home/begin/code/subd/data/AIScenes.json";
    let twitch_user_access_token =
        env::var("TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN").unwrap();

    let contents = fs::read_to_string(file_path).expect("Can read file");
    let ai_scenes: ai_scenes::AIScenes =
        serde_json::from_str(&contents.clone()).map_err(|e| e.to_string())?;
    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;
    let twitch_reward_client: HelixClient<reqwest::Client> = HelixClient::new();
    let token = UserToken::from_existing(
        &reqwest,
        twitch_user_access_token.into(),
        None,
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    let reward_manager = rewards::RewardManager::new(
        &twitch_reward_client,
        &token,
        &broadcaster_id,
    );

    let random = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..ai_scenes.scenes.len())
    };
    let random_scene = &ai_scenes.scenes[random];
    let title = &random_scene.reward_title;

    // If we don't have a reward for that Thang
    let reward_res = twitch_rewards::find_by_title(&pool, title.to_string())
        .await
        .map_err(|e| e.to_string())?;

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
    .map_err(|e| e.to_string())?;

    println!("Update: {:?}", update);

    let msg = format!(
        "Flash Sale! {} - New Low Price! {}",
        reward_res.title, flash_cost
    );
    let _ = send_message(&twitch_client, msg).await;

    Ok(())
}
