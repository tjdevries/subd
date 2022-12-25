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
use anyhow::{bail, Result};
use obws;
use obws::requests::scene_items::Scale;
use obws::Client as OBSClient;
// use rand::rngs::StdRng;
// use rand::thread_rng;
// use rand::Rng;
// use rand::SeedableRng;
use std::fs;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;

pub async fn handle_obs_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    // This is because Begin doesn't understand Rust
    let default_source = String::from(obs::DEFAULT_SOURCE);

    // We try and do some parsing on every command here
    // These may not always be what we want, but they are sensible
    // defaults used by many commands
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);
    // We try and do some parsing on every command here
    // These may not always be what we want, but they are sensible
    // defaults used by many commands let source: &str = splitmsg.get(1).unwrap_or(&default_source);
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

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    // we need to figure a way to look up the defaults per command
    // because they could be different types
    // for now we are going to try and have them be the same
    // let filter_setting_name = splitmsg.get(2).map_or("", |x| x.as_str());

    match splitmsg[0].as_str() {
        // This needs to be Mod only
        "!implicit" => {
            let _ =
                twitch_stream_state::update_implicit_soundeffects(true, &pool)
                    .await;
            Ok(())
        }
        "!random" => {
            // Not Ideal
            let contents = fs::read_to_string("data/voices.json").unwrap();
            let voices: Vec<uberduck::Voice> =
                serde_json::from_str(&contents).unwrap();

            // let mut rng = thread_rng();
            // let mut rng: StdRng = SeedableRng::from_entropy();
            // let random_index = rng.gen_range(0..voices.len());

            uberduck::use_random_voice(msg.contents.clone(), msg.user_name)
                .await?;
            Ok(())
        }

        // "!set_character" => Ok(()),
        "!set_voice" => {
            let default_voice = "brock_samson".to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            uberduck::set_voice(
                voice.to_string(),
                msg.user_name.to_string(),
                pool,
            )
            .await?;
            Ok(())
        }

        "!voice" => {
            let default_voice = "slj".to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);
            uberduck::talk_in_voice(
                msg.contents.clone(),
                voice.to_string(),
                msg.user_name,
                tx,
            )
            .await?;
            Ok(())
        }

        "!soundboard_text" => {
            move_transition_bootstrap::create_soundboard_text(obs_client)
                .await?;
            Ok(())
        }
        "!durf" => Ok(()),
        // ================== //
        // Stream Characters //
        // ================== //
        "!character" => {
            stream_character::create_new_obs_character(obs_client).await?;
            Ok(())
        }

        // ===============================================================================================

        // ================== //
        // Scrolling Sources //
        // ================== //

        // !scroll SOURCE SCROLL_SETTING SPEED DURATION (in milliseconds)
        // !scroll begin x 5 300
        //
        // TODO: Stop using handle_user_input
        "!scroll" => {
            let default_filter_setting_name = String::from("speed_x");

            // This is ok, because we have a different default
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);

            let filter_setting_name: String = match filter_setting_name.as_str()
            {
                "x" => String::from("speed_x"),
                "y" => String::from("speed_y"),
                _ => default_filter_setting_name,
            };

            println!("Starting to Scroll: {} {}", source, filter_setting_name);

            // TODO: THIS 2 is SUPERFLUOUS!!!
            // WE SHOULD RE-WRITE THIS METHOD NOT TO USE IT
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

        // ============ //
        // Blur Sources //
        // ============ //
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
            .await?;

            // }

            Ok(())
        }

        // Update to take in 2 as a const
        "!noblur" | "!unblur" => {
            if msg.roles.is_twitch_mod() {
                println!("WE GOT A MOD OVER HERE");
                move_transition::update_and_trigger_move_value_filter(
                    source,
                    obs::DEFAULT_BLUR_FILTER_NAME,
                    "Filter.Blur.Size",
                    0.0,
                    5000,
                    2,
                    &obs_client,
                )
                .await?;
            }
            Ok(())
        }

        // =============== //
        // Scaling Sources //
        // =============== //
        "!grow" | "!scale" => {
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
            obs_source::trigger_grow(
                &scene,
                source,
                &base_scale,
                x,
                y,
                &obs_client,
            )
            .await
        }

        // ====================== //
        // 3D Transforming Sources//
        // ====================== //
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

        "!hide" => obs_source::hide_sources(obs::MEME_SCENE, &obs_client).await,
        "!show" => {
            obs_source::set_enabled(obs::MEME_SCENE, source, true, &obs_client)
                .await
        }

        // ==================================================================
        // == 3D Filter =====================================================
        // ==================================================================
        "!def_ortho" => {
            stream_fx::default_ortho(source, duration, &obs_client).await
        }
        "!ortho" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

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
        // Perspective
        // Corner Pin
        // Orthographic

        // !3d SOURCE FILTER_NAME FILTER_VALUE DURATION
        // !3d begin Rotation.Z 3600 5000
        //
        // TODO: This is NOT Working!
        "!3d" => {
            // If we don't at least have a filter_name, we can't proceed
            if splitmsg.len() < 3 {
                bail!("We don't have a filter name, can't proceed");
            }

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

        // ============== //
        // Moving Sources //
        // ============== //
        "!move" => {
            // TODO: Look at this fanciness
            //       cafce25: if let [source, x, y, ..] = splitmsg {...}

            println!("!move {} {}", scene, source);

            if splitmsg.len() > 3 {
                let source = splitmsg[1].as_str();
                let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
                let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);

                obs_source::move_source(&scene, source, x, y, &obs_client).await
            } else {
                Ok(())
            }
        }

        // TODO: I'd like one-for every corner
        "!tr" => move_transition_effects::top_right(source, &obs_client).await,

        "!bl" => {
            move_transition_effects::bottom_right(source, &obs_client).await
        }

        // ================ //
        // Compound Effects //
        // ================ //
        "!norm" => obs_combo::norm(&source, &obs_client).await,

        "!follow" => {
            let scene = obs::DEFAULT_SCENE;
            let leader = splitmsg.get(1).unwrap_or(&default_source);
            let source = leader;

            obs_combo::follow(source, scene, leader, &obs_client).await
        }
        "!staff" => obs_combo::staff(obs::DEFAULT_SOURCE, &obs_client).await,

        // =============================== //
        // Create Scenes, Sources, Filters //
        // =============================== //
        "!create_source" => {
            let new_scene: obws::requests::scene_items::CreateSceneItem =
                obws::requests::scene_items::CreateSceneItem {
                    scene: obs::DEFAULT_SCENE,
                    source: &source,
                    enabled: Some(true),
                };

            // TODO: Why is this crashing???
            obs_client.scene_items().create(new_scene).await?;

            Ok(())
        }

        // TEMP: This is for temporary testing!!!!
        "!split" => {
            bootstrap::create_split_3d_transform_filters(source, &obs_client)
                .await
        }

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            bootstrap::create_filters_for_source(source, &obs_client).await
        }

        // TODO: Take in Scene
        "!source" => {
            obs_source::print_source_info(
                source,
                obs::DEFAULT_SCENE,
                &obs_client,
            )
            .await
        }

        "!outline" => {
            let source = splitmsg[1].as_str();
            sdf_effects::outline(source, &obs_client).await
        }

        // ====================== //
        // Show / Hide Subscenes //
        // ====================== //
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

        // ==================== //
        // Change Scenes in OBS //
        // ==================== //
        // Rename These Commands
        "!chat" => obs_hotkeys::trigger_hotkey("OBS_KEY_L", &obs_client).await,

        "!code" => obs_hotkeys::trigger_hotkey("OBS_KEY_H", &obs_client).await,
        _ => Ok(()),
    }
}
