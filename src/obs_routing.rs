use crate::bootstrap;
use crate::move_transition;
use crate::obs;
use crate::obs_combo;
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
use rand::thread_rng;
use rand::Rng;
use std::fs;
use std::path::Path;
use subd_types::TransformOBSTextRequest;
use subd_types::UberDuckRequest;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;

pub const DEFAULT_SCENE: &str = "Primary";
pub const MEME_SCENE: &str = "memes";
pub const DEFAULT_SOURCE: &str = "begin";

// This should be here
// const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";
pub async fn handle_obs_commands(
    tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    // This is because Begin doesn't understand Rust
    let default_source = String::from(DEFAULT_SOURCE);

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

    let scene = match obs::find_scene(source).await {
        Ok(scene) => scene.to_string(),
        Err(_) => MEME_SCENE.to_string(),
    };

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    // we need to figure a way to look up the defaults per command
    // because they could be different types
    // for now we are going to try and have them be the same
    // let filter_setting_name = splitmsg.get(2).map_or("", |x| x.as_str());
    //
    match splitmsg[0].as_str() {
        // This needs to be Mod only
        "!implicit" => {
            twitch_stream_state::update_implicit_soundeffects(true, &pool)
                .await?;
            Ok(())
        }
        // Need to finish using this
        "!random" => {
            let contents = fs::read_to_string("data/voices.json").unwrap();
            let voices: Vec<uberduck::Voice> =
                serde_json::from_str(&contents).unwrap();
            let mut rng = thread_rng();
            let random_index = rng.gen_range(0..voices.len());
            let random_voice = &voices[random_index];

            println!("Random Voice Chosen: {:?}", random_voice);

            let spoken_string = msg.contents.clone().replace("!random", "");

            let mut seal_text = spoken_string.clone();
            let spaces: Vec<_> = spoken_string.match_indices(" ").collect();
            let line_length_modifier = 20;
            let mut line_length_limit = 20;
            for val in spaces.iter() {
                if val.0 > line_length_limit {
                    seal_text.replace_range(val.0..=val.0, "\n");
                    line_length_limit =
                        line_length_limit + line_length_modifier;
                }
            }

            let voice_text = spoken_string.clone();

            let _ = tx.send(Event::TransformOBSTextRequest(
                TransformOBSTextRequest {
                    message: random_voice.name.clone(),
                    text_source: "Soundboard-Text".to_string(),
                },
            ));

            let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                voice: random_voice.name.clone(),
                message: seal_text,
                voice_text,
                username: msg.user_name,
                source: None,
            }));
            Ok(())
        }

        "!set_character" => {
            // let default_character = "Seal".to_string();
            // let character: &str = splitmsg.get(1).unwrap_or(&default_character);

            // // We need to look up first
            // // and not change other things actually
            // let model =
            //     stream_character::user_stream_character_information::Model {
            //         username: msg.user_name.clone(),
            //         voice: voice.to_string(),
            //         obs_character: "Seal".to_string(),
            //         random: false,
            //     };

            // model.save(pool).await?;

            Ok(())
        }

        "!set_voice" => {
            let default_voice = "brock_samson".to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);

            // We need to look up first
            // and not change other things actually
            let model =
                stream_character::user_stream_character_information::Model {
                    username: msg.user_name.clone(),
                    voice: voice.to_string(),
                    obs_character: "Seal".to_string(),
                    random: false,
                };

            model.save(pool).await?;

            Ok(())
        }

        "!voice" => {
            let default_voice = "slj".to_string();
            let voice: &str = splitmsg.get(1).unwrap_or(&default_voice);

            let spoken_string = msg
                .contents
                .clone()
                .replace(&format!("!voice {}", &voice), "");

            if spoken_string == "" {
                return Ok(());
            }

            let mut seal_text = spoken_string.clone();
            let spaces: Vec<_> = spoken_string.match_indices(" ").collect();
            let line_length_modifier = 20;
            let mut line_length_limit = 20;
            for val in spaces.iter() {
                if val.0 > line_length_limit {
                    seal_text.replace_range(val.0..=val.0, "\n");
                    line_length_limit =
                        line_length_limit + line_length_modifier;
                }
            }

            // let voice_text = msg.contents.to_string();
            let voice_text = spoken_string.clone();
            // I need to get some logic to grab all the text after x position
            println!("We trying for the voice: {} - {}", voice, voice_text);
            // I can get that from user name
            let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
                voice: voice.to_string(),
                message: seal_text,
                voice_text,
                username: msg.user_name,
                source: None,
            }));
            Ok(())
        }

        "!soundboard_text" => {
            let scene = "Characters";

            // let font_flags = obws::common::FontFlags{ }
            let font = obws::requests::custom::source_settings::Font {
                face: "Arial",
                size: 256,
                style: "Regular",
                ..Default::default()
            };

            // So these are fugazi???
            // I expect these colors to be something
            let color1 = rgb::RGBA::new(255, 0, 132, 0);
            let color2 = rgb::RGBA::new(0, 3, 255, 1);

            let text_settings =
                obws::requests::custom::source_settings::TextFt2SourceV2 {
                    outline: true,
                    drop_shadow: true,
                    text: "SoundBoard!",
                    color1,
                    color2,
                    font,
                    custom_width: 5,
                    log_lines: 5,
                    word_wrap: false,
                    ..Default::default() // We might want to experiment from file
                };

            let text_source_name = "Soundboard-Text";
            let _ = obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &text_source_name,
                    kind: "text_ft2_source_v2",
                    settings: Some(text_settings),
                    enabled: Some(true),
                })
                .await;

            let filter_name = "TransformSoundBoard-text";
            let move_text_filter = move_transition::MoveTextFilter {
                setting_name: "text".to_string(),
                setting_text: "Ok NOW".to_string(),
                value_type: 5,
                ..Default::default()
            };
            let new_filter = obws::requests::filters::Create {
                source: &text_source_name,
                filter: &filter_name,
                kind: "move_value_filter",
                settings: Some(move_text_filter),
            };
            if let Err(err) = obs_client.filters().create(new_filter).await {
                println!("Error Creating Filter: {filter_name} | {:?}", err);
            };

            Ok(())
        }
        "!durf" => Ok(()),
        // ================== //
        // Stream Characters //
        // ================== //
        "!character" => {
            let scene = "Characters";

            // let base_source = "Seal";
            // let base_source = "Birb";
            // let base_source = "Kevin";
            let base_source = "Randall";
            // let base_source = "Teej";
            // let base_source = "ArtMatt";

            let filename = format!(
                "/home/begin/stream/Stream/StreamCharacters/{}.png",
                base_source
            );

            // TODO: We need to pull in this source
            let image_source =
                obws::requests::custom::source_settings::ImageSource {
                    file: Path::new(&filename),
                    ..Default::default()
                };
            let _ = obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &base_source,
                    kind: "image_source",
                    settings: Some(image_source),
                    enabled: Some(true),
                })
                .await;

            let speech_bubble =
                obws::requests::custom::source_settings::ImageSource {
                file: Path::new("/home/begin/stream/Stream/StreamCharacters/speech_bubble.png"),
                    ..Default::default()
                };
            let speech_source_name = format!("{}-speech_bubble", base_source);
            let _ = obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &speech_source_name,
                    kind: "image_source",
                    settings: Some(speech_bubble),
                    enabled: Some(true),
                })
                .await;

            // let font_flags = obws::common::FontFlags{ }
            let font = obws::requests::custom::source_settings::Font {
                face: "Arial",
                size: 256,
                style: "Regular",
                ..Default::default()
            };

            // So these are fugazi???
            // I expect these colors to be something
            let color1 = rgb::RGBA::new(255, 0, 132, 0);
            let color2 = rgb::RGBA::new(0, 3, 255, 1);

            let text_settings =
                obws::requests::custom::source_settings::TextFt2SourceV2 {
                    outline: true,
                    drop_shadow: true,
                    text: "This Rules we are doing something!",
                    color1,
                    color2,
                    font,
                    custom_width: 5,
                    log_lines: 5,
                    word_wrap: false,
                    ..Default::default() // We might want to experiment from file
                };

            let text_source_name = format!("{}-text", base_source);
            let _ = obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &text_source_name,
                    kind: "text_ft2_source_v2",
                    settings: Some(text_settings),
                    enabled: Some(true),
                })
                .await;

            // ======================================================
            // This is creating the Text Transform Filter
            // Not Sure of This Name
            // We just need a better name
            // Create Move-Value for 3D Transform Filter
            let filter_name = format!("Transform{}-text", base_source);
            let move_text_filter = move_transition::MoveTextFilter {
                setting_name: "text".to_string(),
                setting_text: "Ok NOW".to_string(),
                value_type: 4,
                ..Default::default()
            };
            let new_filter = obws::requests::filters::Create {
                source: &text_source_name,
                filter: &filter_name,
                kind: "move_value_filter",
                settings: Some(move_text_filter),
            };
            if let Err(err) = obs_client.filters().create(new_filter).await {
                println!("Error Creating Filter: {filter_name} | {:?}", err);
            };

            // move_transition_hide_source.json
            // move_transition_hide_speech_bubble.json
            // move_transition_hide_text.json
            // move_transition_show_speech_bubble.json
            // move_transition_show_text.json
            //
            // ======================================================
            let file_path = "/home/begin/code/subd/obs_data/move_transition_show_source.json";
            let filter_name = format!("Show{}", base_source);
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &base_source,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;

            let filter_name = format!("Hide{}", base_source);
            let file_path = "/home/begin/code/subd/obs_data/move_transition_hide_source.json";
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &base_source,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;

            let filter_name = format!("Show{}-text", base_source);
            let file_path =
                "/home/begin/code/subd/obs_data/move_transition_show_text.json";
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &text_source_name,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;

            let filter_name = format!("Hide{}-text", base_source);
            let file_path =
                "/home/begin/code/subd/obs_data/move_transition_hide_text.json";
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &text_source_name,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;

            let filter_name = format!("Show{}-speech_bubble", base_source);
            let file_path =
                "/home/begin/code/subd/obs_data/move_transition_show_speech_bubble.json";
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &speech_source_name,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;

            let filter_name = format!("Hide{}-speech_bubble", base_source);
            let file_path =
                "/home/begin/code/subd/obs_data/move_transition_hide_speech_bubble.json";
            let _ = move_transition::create_move_source_filter_from_file(
                scene,
                &speech_source_name,
                &filter_name,
                file_path,
                &obs_client,
            )
            .await;
            Ok(())
        }

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
            obs::handle_user_input(
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

        // This shit is annoying
        // I almost want to divide it into 3 commands
        // based on Camera Type
        // and we have all 3
        // that might be too much
        // but i also might be exactly what we want
        // only spin is wonky
        // Should also add !spinz
        "!spin" | "!spinx" | "spiny" => {
            // HMMMMM
            let default_filter_setting_name = String::from("z");
            let filter_setting_name =
                splitmsg.get(2).unwrap_or(&default_filter_setting_name);

            move_transition::spin(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!hide" => obs_source::hide_sources(MEME_SCENE, &obs_client).await,
        "!show" => {
            obs_source::set_enabled(MEME_SCENE, source, true, &obs_client).await
        }
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

            move_transition::trigger_3d(
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
        "!tr" => move_transition::top_right(source, &obs_client).await,

        "!bl" => move_transition::bottom_right(source, &obs_client).await,

        // ================ //
        // Compound Effects //
        // ================ //
        "!norm" => obs_combo::norm(&source, &obs_client).await,

        "!follow" => {
            let scene = DEFAULT_SCENE;
            let leader = splitmsg.get(1).unwrap_or(&default_source);
            let source = leader;

            obs_combo::follow(source, scene, leader, &obs_client).await
        }
        "!staff" => obs_combo::staff(DEFAULT_SOURCE, &obs_client).await,

        // =============================== //
        // Create Scenes, Sources, Filters //
        // =============================== //
        "!create_source" => {
            let new_scene: obws::requests::scene_items::CreateSceneItem =
                obws::requests::scene_items::CreateSceneItem {
                    scene: DEFAULT_SCENE,
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

        // ========================== //
        // Show Info About OBS Setup  //
        // ========================== //
        // "!filter" => {
        //     let (_command, words) = msg.message_text.split_once(" ").unwrap();

        //     // TODO: Handle this error
        //     let details =
        //         print_filter_info(&source, words, &obs_client)
        //             .await?;
        //     client
        //         .say(twitch_username.clone(), format!("{:?}", details))
        //         .await
        // }

        // TODO: Take in Scene
        "!source" => {
            obs::print_source_info(source, DEFAULT_SCENE, &obs_client).await
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
                DEFAULT_SCENE,
                MEME_SCENE,
                true,
                &obs_client,
            )
            .await
        }

        "!nomemes" | "!nojokes" | "!work" => {
            obs_source::set_enabled(
                DEFAULT_SCENE,
                MEME_SCENE,
                false,
                &obs_client,
            )
            .await
        }

        // ==================== //
        // Change Scenes in OBS //
        // ==================== //
        // Rename These Commands
        "!chat" => obs::trigger_hotkey("OBS_KEY_L", &obs_client).await,

        "!code" => obs::trigger_hotkey("OBS_KEY_H", &obs_client).await,

        _ => Ok(()),
    }
}
