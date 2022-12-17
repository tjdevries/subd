use crate::move_transition;
use crate::obs;
use anyhow::{bail, Result};
use obws;
use obws::requests::scene_items::Scale;
use obws::Client as OBSClient;
use std::path::Path;
// use serde::{Deserialize, Serialize};
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;

const DEFAULT_SCENE: &str = "Primary";
const MEME_SCENE: &str = "memes";
const DEFAULT_SOURCE: &str = "begin";

// This should be here
// const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";

pub async fn handle_obs_commands(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
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

    // NOTE: If we want to extract values like filter_setting_name and filter_value
    // we need to figure a way to look up the defaults per command
    // because they could be different types
    // for now we are going to try and have them be the same
    // let filter_setting_name = splitmsg.get(2).map_or("", |x| x.as_str());
    //
    // the source got fucked up some how

    match splitmsg[0].as_str() {
        // ================== //
        // Stream Characters //
        // ================== //

        // Creating Filters for Stream Characters
        // let filter_name = format!("Move_Source_Home_{}", source);
        "!character" => {
            // So we take the source from what's passed in
            // We actually need to create the source as well
            let scene = "Characters";
            let base_source = "Birb";

            // We should really get this into it's own Method
            // let filter_name = "Show-Kevin";

            // We need create 2 functions
            // for triggering the show and hide of the characters
            // This could be the base image
            let image_source =
                obws::requests::custom::source_settings::ImageSource {
                    file: Path::new(
                        "/home/begin/stream/Stream/StreamCharacters/birb.png",
                    ),
                    ..Default::default()
                };
            obs_client
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
            obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &speech_source_name,
                    kind: "image_source",
                    settings: Some(speech_bubble),
                    enabled: Some(true),
                })
                .await;

            // Then we need the Text

            // We got to figure out how to get some recent early positions
            // WE should try and create all these in a position
            let text_settings =
                obws::requests::custom::source_settings::TextFt2SourceV2 {
                    outline: true,
                    drop_shadow: true,
                    text: "THIS RULESSSSS WE RULE!!!!",
                    // antialiasing: todo!(),
                    // font: todo!(),
                    // from_file: todo!(),
                    // log_lines: todo!(),
                    // log_mode: todo!(),
                    // rgb::RGBA<u8>
                    // color1: 4286578517,
                    // color2: 4278190335,
                    // "font": {
                    //     "face": "Arial",
                    //     "flags": 0,
                    //     "size": 256,
                    //     "style": "Regular"
                    // },
                    // text_file: todo!(),
                    // word_wrap: todo!(),
                    ..Default::default()
                };
            let text_source_name = format!("{}-text", base_source);
            obs_client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene,
                    input: &text_source_name,
                    kind: "text_ft2_source_v2",
                    settings: Some(text_settings),
                    enabled: Some(true),
                })
                .await;

            let filter_name = format!("{}-text", base_source);
            obs::create_move_source_filters(
                "Characters",
                &text_source_name,
                &filter_name,
                &obs_client,
            )
            .await;

            // Not Sure of This Name
            // We just need a better name
            // Create Move-Value for 3D Transform Filter
            let filter_name = format!("Transform{}-text", base_source);
            let move_text_filter = move_transition::MoveTextFilter {
                setting_name: "text".to_string(),
                setting_text: "Ok NOW".to_string(),
                value_type: 5,
                // Something is wrong
                // might be 5
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
            // "name": "chief-keef",
            // Now I just need o use this
            // pub setting_name: String,
            // #[serde(rename = "value_type")]
            // pub value_type: u32,
            // #[serde(rename = "setting_text")]
            // pub setting_text: String,
            // We still need the Filter on Text Transform
            // Which we haven't made yet

            // Then we need to create 6 Filters
            let filter_name = format!("Show{}", base_source);
            obs::create_move_source_filters(
                "Characters",
                &base_source,
                &filter_name,
                &obs_client,
            )
            .await;
            let filter_name = format!("Hide{}", base_source);
            obs::create_move_source_filters(
                "Characters",
                &base_source,
                &filter_name,
                &obs_client,
            )
            .await;

            let filter_name = format!("Show{}-text", base_source);
            obs::create_move_source_filters(
                "Characters",
                &text_source_name,
                &filter_name,
                &obs_client,
            )
            .await;
            let filter_name = format!("Hide{}-text", base_source);
            obs::create_move_source_filters(
                "Characters",
                &text_source_name,
                &filter_name,
                &obs_client,
            )
            .await;

            // Doubling for Hide is easy
            // We just need to know how to get get starting positions
            let filter_name = format!("Show{}-speech_bubble", base_source);
            obs::create_move_source_filters(
                "Characters",
                &speech_source_name,
                &filter_name,
                &obs_client,
            )
            .await;
            let filter_name = format!("Hide{}-speech_bubble", base_source);
            obs::create_move_source_filters(
                "Characters",
                &speech_source_name,
                &filter_name,
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

            // TODO: THIS 2 is SUPERFLUOUS!!!
            // WE SHOULD RE-WRITE THIS METHOD NOT TO USE IT
            obs::handle_user_input(
                source,
                obs::DEFAULT_SCROLL_FILTER_NAME,
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

            // msg.roles.is_twitch_mod()
            // msg.roles.is_twitch_founder()
            // msg.roles.is_twitch_staff()
            // msg.roles.is_twitch_sub()
            // if msg.roles.is_twitch_vip() {

            // So maybe the source is wrong
            // maybe the DEFAULT_MOVE_BLUR_FILTER_NAME name is wrong
            //
            // the 2 is also problematic
            // and we aren't pull in the duration
            obs::update_and_trigger_move_value_filter(
                source,
                obs::DEFAULT_BLUR_FILTER_NAME,
                "Filter.Blur.Size",
                filter_value,
                300,
                // 5000, // duration
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
                obs::update_and_trigger_move_value_filter(
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
            obs::trigger_grow(source, &base_scale, x, y, &obs_client).await
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

            obs::spin(
                source,
                filter_setting_name,
                filter_value,
                duration,
                &obs_client,
            )
            .await
        }

        "!hide" => obs::hide_sources(MEME_SCENE, &obs_client).await,
        "!show" => {
            obs::set_enabled(MEME_SCENE, source, true, &obs_client).await
        }
        "!def_ortho" => obs::default_ortho(source, duration, &obs_client).await,
        "!ortho" => {
            if splitmsg.len() < 3 {
                return Ok(());
            };

            let filter_setting_name = &splitmsg[2];

            obs::trigger_ortho(
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

            obs::trigger_ortho(
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

            obs::trigger_ortho(
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

            obs::trigger_3d(
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
            if splitmsg.len() > 3 {
                let source = splitmsg[1].as_str();
                let x: f32 = splitmsg[2].trim().parse().unwrap_or(0.0);
                let y: f32 = splitmsg[3].trim().parse().unwrap_or(0.0);

                obs::move_source(source, x, y, &obs_client).await
            } else {
                Ok(())
            }
        }

        // TODO: I'd like one-for every corner
        "!tr" => obs::top_right(source, &obs_client).await,

        "!bl" => obs::bottom_right(source, &obs_client).await,

        // ================ //
        // Compound Effects //
        // ================ //
        "!norm" => obs::norm(&source, &obs_client).await,

        "!follow" => {
            let scene = DEFAULT_SCENE;
            let leader = splitmsg.get(1).unwrap_or(&default_source);
            let source = leader;

            obs::follow(source, scene, leader, &obs_client).await
        }
        "!staff" => obs::staff(DEFAULT_SOURCE, &obs_client).await,

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
            obs::create_split_3d_transform_filters(source, &obs_client).await
        }

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            obs::create_filters_for_source(source, &obs_client).await
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
            obs::outline(source, &obs_client).await
        }

        // ====================== //
        // Show / Hide Subscenes //
        // ====================== //
        "!memes" => {
            obs::set_enabled(DEFAULT_SCENE, MEME_SCENE, true, &obs_client).await
        }

        "!nomemes" | "!nojokes" | "!work" => {
            obs::set_enabled(DEFAULT_SCENE, MEME_SCENE, false, &obs_client)
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
