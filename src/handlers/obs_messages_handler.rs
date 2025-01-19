use crate::chat_parser::parser;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use num_traits::ToPrimitive;
use obs_3d_filter::orthographic::ThreeDTransformOrthographic;
use obs_3d_filter::perspective::ThreeDTransformPerspective;
use obs_3d_filter::CameraMode;
use obs_move_transition;
use obs_move_transition::duration;
use obs_move_transition::update_and_trigger_move_value_for_source;
use obs_service::obs_scenes;
use obs_service::obs_source;
use obws;
use obws::requests::sources::SourceId;
use obws::Client as OBSClient;
use rodio::*;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

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
                .split(' ')
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
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let default_source = subd_types::consts::get_default_obs_source();
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let is_mod = msg.roles.is_twitch_mod();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let _duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
    let _scene = obs_scenes::find_scene(source)
        .await
        .unwrap_or(subd_types::consts::get_meme_scene());
    let command = splitmsg[0].as_str();

    let (scene, source) = match source {
        "begin" => ("Begin", "begin"),
        "bogan" => ("AIAssets", source),
        "randall" => ("TestScene", source),
        _ => ("Memes", source),
    };

    let _ = match command {
        "!hotkey" => {
            log::info!("HOTKEY TIME");
            // let hotkey_event =
            //     Event::TriggerHotkeyRequest(subd_types::TriggerHotkeyRequest {
            //         hotkey: "E".to_string(), // The hotkey name/ID to trigger
            //     });
            // // Send the event
            // tx.send(hotkey_event)?;

            // let hotkey = obws::requests::hotkeys::KeyModifiers;
            obs_service::obs_hotkeys::trigger_hotkey("H", &obs_client).await?;

            // This works
            // let _ =
            //     obs_source::set_enabled("Scene 2", "Image", false, obs_client)
            //         .await;
            //
            Ok(())
        }
        "!stroke" => {
            let stroke_value = splitmsg
                .get(1)
                .map(|v| v.parse::<f32>().unwrap_or(99.9))
                .unwrap_or(99.9);
            let filter = "Move_Outline";
            let source = "BeginOutline1";

            if let Err(e) = update_and_trigger_move_value_for_source(
                obs_client,
                source,
                filter,
                "stroke_size",
                // "stroke_offset",
                stroke_value,
            )
            .await
            {
                println!("Error: {:?}", e);
            }
            Ok(())
        }

        "!find" => {
            let filter_name = format!("Move_{}", source);
            obs_move_transition::find_source(
                scene,
                source,
                filter_name,
                obs_client,
            )
            .await
        }

        "!move" => {
            let filter_name = format!("Move_{}", source);
            let x = splitmsg.get(2).map(|v| v.parse::<f32>().unwrap_or(100.0));
            let y = splitmsg.get(3).map(|v| v.parse::<f32>().unwrap_or(100.0));
            let res = obs_move_transition::move_source(
                scene,
                source,
                filter_name,
                true,
                x,
                y,
                None,
                None,
                None,
                obs_client,
            )
            .await;
            if let Err(err) = res {
                println!("Error: {:?}", err);
            }
            Ok(())
        }

        "!wide" => {
            println!("Wide Time!");
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = parser::default_wide_args();
            let req =
                parser::build_wide_request(meat_of_message, &arg_positions)?;
            let settings = ThreeDTransformOrthographic {
                scale_x: Some(300.0),
                camera_mode: CameraMode::Orthographic,
                ..Default::default()
            };
            let d = duration::EasingDuration::new(req.duration as i32);
            let _ = obs_move_transition::update_and_trigger_filter(
                obs_client,
                &req.source,
                "3D-Transform-Orthographic",
                settings,
                d,
            )
            .await;
            return Ok(());
        }

        "!nerd" => {
            let settings = ThreeDTransformPerspective::builder()
                .scale_x(Some(125.3))
                .scale_y(Some(140.6))
                .position_y(Some(40.0))
                .rotation_x(Some(-51.4))
                .field_of_view(Some(90.0))
                .build();

            let d = duration::EasingDuration::new(3000);

            let _ = obs_move_transition::update_and_trigger_filter(
                obs_client,
                source,
                "3D-Transform-Perspective",
                settings,
                d,
            )
            .await;
            Ok(())
        }

        // !update_meme SOURCE X Y
        "!update_meme" => {
            if !is_mod {
                return Ok(());
            }
            let x = splitmsg
                .get(2)
                .ok_or(anyhow!("Error Fetching X to update_meme"))?
                .parse::<f32>()?;

            let y =
                splitmsg.get(2).map_or(x, |v| v.parse::<f32>().unwrap_or(x));

            let _ = obs_source::update_obs_source_position(
                pool,
                source.to_string(),
                x,
                y,
            )
            .await;
            Ok(())
        }

        "!norm" => {
            let filters = vec![
                "Default_3D-Transform-Orthographic",
                "Default_3D-Transform-Perspective",
                "Default_3D-Transform-CornerPin",
            ];
            for filter in filters {
                let filter_enabled = obws::requests::filters::SetEnabled {
                    source: SourceId::Name(source),
                    filter,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }

            let res =
                obs_source::get_obs_source(pool, source.to_string()).await?;

            let _scale = res
                .scale
                .to_f32()
                .ok_or(anyhow!("Error converting scale to f32"))?;

            let position_x = res
                .position_x
                .to_f32()
                .ok_or(anyhow!("Error converting position_x to f32"))?;
            let position_y = res
                .position_y
                .to_f32()
                .ok_or(anyhow!("Error converting position_y to f32"))?;
            let scene = res.scene;

            let duration = 3000;
            let d = duration::EasingDuration::new(duration);

            obs_move_transition::move_source_in_scene_x_and_y(
                obs_client, &scene, source, position_x, position_y, d,
            )
            .await?;

            Ok(())
        }

        "!chad" => {
            // This should be a new
            let settings = ThreeDTransformPerspective::builder()
                .scale_x(Some(217.0))
                .scale_y(Some(200.0))
                .rotation_x(Some(50.0))
                .field_of_view(Some(108.0))
                .build();
            let d = duration::EasingDuration::new(3000);
            let _ = obs_move_transition::update_and_trigger_filter(
                obs_client,
                source,
                "3D-Transform-Perspective",
                settings,
                d,
            )
            .await;
            Ok(())
        }

        // ===========================================
        // == Scaling Sources
        // ===========================================
        "!grow" | "!scale" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = parser::default_move_or_scale_args();
            let req = parser::build_chat_move_source_request(
                meat_of_message,
                &arg_positions,
            );
            dbg!(&req);
            let filter_name = format!("Move_{}", source);
            let x = splitmsg
                .get(2)
                .map(|v| v.parse::<f32>().unwrap_or(2.0))
                .unwrap_or(2.0);
            let y = splitmsg
                .get(3)
                .map(|v| v.parse::<f32>().unwrap_or(x))
                .unwrap_or(x);
            let res = obs_move_transition::scale_source(
                scene,
                source,
                filter_name,
                x,
                y,
                obs_client,
            )
            .await;
            if let Err(err) = res {
                println!("Error: {:?}", err);
            }
            // Add Scale code
            Ok(())
        }

        "!alex" => {
            let source = "alex";
            let scene = "Memes";
            let arg_positions = &parser::default_move_or_scale_args()[1..];
            let req = parser::build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );
            let d = duration::EasingDuration::new(req.duration as i32);
            obs_move_transition::move_source_in_scene_x_and_y(
                obs_client, scene, source, req.x, req.y, d,
            )
            .await
        }

        "!begin" => {
            let source = "begin";
            let scene = subd_types::consts::get_primary_camera_scene();
            let arg_positions = &parser::default_move_or_scale_args()[1..];
            let req = parser::build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );
            let d = duration::EasingDuration::new(
                req.duration.try_into().unwrap_or(3000),
            );
            obs_move_transition::move_source_in_scene_x_and_y(
                obs_client, &scene, source, req.x, req.y, d,
            )
            .await
        }
        //
        "!filter" => {
            let default_filter_name = "3D-Transform-Perspective".to_string();
            let filter: &str = splitmsg.get(1).unwrap_or(&default_filter_name);
            let filter_details = obs_client
                .filters()
                .get(SourceId::Name("begin"), filter)
                .await?;
            println!("------------------------");
            println!("\n\tFilter Settings: {:?}", filter_details);
            println!("------------------------");
            Ok(())
        }

        "!twirl" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = &parser::default_twirl_args();
            let req = parser::build_chat_twirl_request(
                meat_of_message,
                arg_positions,
            );
            let settings = ThreeDTransformOrthographic {
                rotation_y: Some(req.rotation_y),
                camera_mode: CameraMode::Orthographic,
                ..Default::default()
            };
            let d = duration::EasingDuration::new(req.duration as i32);
            let _ = obs_move_transition::update_and_trigger_filter(
                obs_client,
                source,
                "3D-Transform-Orthographic",
                settings,
                d,
            )
            .await;
            Ok(())
        }

        "!rot" => {
            let filter_name = format!("Move_{}", source);
            let z = splitmsg
                .get(2)
                .map(|v| v.parse::<f32>().unwrap_or(360.0))
                .unwrap_or(360.0);
            let res = obs_move_transition::rot_source(
                scene,
                source,
                filter_name,
                z,
                obs_client,
            )
            .await;
            if let Err(err) = res {
                println!("Error: {:?}", err);
            }
            Ok(())
        }

        // Examples:
        //           !spin 1080 18000 ease-in-and-out cubic
        //
        // !spin SPIN_AMOUNT DURATION EASING-TYPE EASING-FUNCTION
        "!spin" | "!spinx" | "spiny" => {
            let arg_positions = &parser::default_spin_args();
            let req = parser::build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );
            let d = duration::EasingDuration::new(req.duration as i32);
            obs_move_transition::spin_source(
                obs_client,
                &req.source,
                req.rotation_z,
                d,
            )
            .await
        }

        // This need to be updated
        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            if _not_beginbot {
                return Ok(());
            }
            let default = "alex".to_string();
            let source: &str = splitmsg.get(1).unwrap_or(&default);

            // These aren't implemented properly
            _ = obs_bootstrap::remove_all_filters(source, obs_client).await;
            obs_bootstrap::create_split_3d_transform_filters(source, obs_client)
                .await
        }

        _ => Ok(()),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use obs_service::obs;
    use serde_json::Value;

    //This is just for printing settings
    #[tokio::test]
    #[ignore]
    async fn test_fetching_filter_settings() {
        let obs_client = obs::create_obs_client().await.unwrap();
        // let filter = "Move0".to_string();
        // let filter_details =
        //     obs_client.filters().get(source, &filter).await.unwrap();
        // println!("------------------------");
        // println!("\n\tFilter Settings: {:?}", filter_details);
        // println!("------------------------");

        let source = "BeginOutline2";
        let scene = "OutlineEffects";
        let item_id = obs_source::find_id(scene, source, &obs_client)
            .await
            .unwrap();

        println!("Item ID: {:?}", item_id);

        // I want to see what this so I can write a struct to Deserialize
        // But I don't know waht the values to be ignored
        // serde::Value
        let settings =
            obs_client.inputs().settings::<Value>(source).await.unwrap();
        println!("------------------------");
        println!("\n\tSource: {:?}", settings);
        println!("------------------------");
    }
}
