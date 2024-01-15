use crate::constants;
use crate::move_transition::models;
use crate::move_transition::move_transition;
use crate::obs::obs_scenes;
use crate::obs::obs_source;
use crate::obs_bootstrap::bootstrap;
use crate::obs_filters;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use num_traits::ToPrimitive;
use obws;
use obws::Client as OBSClient;
use rodio::*;
use std::collections::HashMap;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

const PRIMARY_CAM_SCENE: &str = "Begin";

pub enum WideArgPosition {
    Source(String),
    X(f32),
    Duration(u64),
}

pub enum ChatArgPosition {
    Source(String),
    X(f32),
    Y(f32),
    RotationX(f32),
    RotationY(f32),
    RotationZ(f32),
    Duration(u64),
    EasingType(String),
    EasingFunction(String),
}

#[derive(Default, Debug)]
pub struct WideRequest {
    source: String,
    _scene: String,
    x: f32,
    duration: u64,
}

// This is used inside of OBS Messages
// It also does more than Move
// This is related to chat, and shouldn't note taht
#[derive(Default, Debug)]
pub struct ChatMoveSourceRequest {
    // I think we are always operating on a source, and sometimes also have a scene?
    pub source: String,
    pub scene: String,

    pub x: f32,
    pub y: f32,
    pub rotation_z: f32,

    // I think we need to learn to flatten this
    pub duration: u64,
    pub easing_type: String,
    pub easing_function: String,
    pub easing_type_index: i32,
    pub easing_function_index: i32,
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
    _tx: &broadcast::Sender<Event>,
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
    let default_source = constants::DEFAULT_SOURCE.to_string();
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);

    let is_mod = msg.roles.is_twitch_mod();
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let _duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
    let _scene = obs_scenes::find_scene(source)
        .await
        .unwrap_or(constants::MEME_SCENE.to_string());
    let command = splitmsg[0].as_str();

    let _ = match command {
        "!wide" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = default_wide_args();
            let req = build_wide_request(meat_of_message, &arg_positions)?;
            let settings =
                obs_filters::three_d_transform::ThreeDTransformOrthographic {
                    scale_x: Some(300.0),
                    camera_mode: (),
                    ..Default::default()
                };
            let d = models::DurationSettings {
                duration: Some(req.duration as i32),
                ..Default::default()
            };
            let _ = move_transition::update_and_trigger_3d_filter(
                &obs_client,
                &req.source,
                "3D-Transform-Orthographic",
                settings,
                d,
            )
            .await;

            // let filter_value = 300.0;
            // let filter_name = "3D-Transform-Orthographic";
            // let filter_setting_name = "Scale.X";
            // let _ = move_transition::trigger_move_value_3d_transform(
            //     &req.source,
            //     filter_name,
            //     filter_setting_name,
            //     filter_value,
            //     req.duration as u32,
            //     obs_client,
            // )
            // .await;

            return Ok(());
        }

        "!nerd" => {
            let settings =
                obs_filters::three_d_transform::ThreeDTransformPerspective {
                    scale_x: Some(125.3),
                    scale_y: Some(140.6),
                    position_y: Some(40.0),
                    rotation_x: Some(-51.4),
                    field_of_view: Some(90.0),
                    camera_mode: (),
                    ..Default::default()
                };
            let d = models::DurationSettings {
                ..Default::default()
            };

            let _ = move_transition::update_and_trigger_3d_filter(
                &obs_client,
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
                &pool,
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
                    source,
                    filter: &filter,
                    enabled: true,
                };
                obs_client.filters().set_enabled(filter_enabled).await?;
            }

            let res =
                obs_source::get_obs_source(&pool, source.to_string()).await?;

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

            // TODO: find proper move:

            let duration = 3000;
            let easing_function_index = 1;
            let easing_type_index = 1;
            let d = models::DurationSettings {
                duration: Some(duration),
                easing_function_index: Some(easing_function_index),
                easing_type_index: Some(easing_type_index),
                ..Default::default()
            };

            let _ = move_transition::move_source_in_scene_x_and_y(
                obs_client, &scene, source, position_x, position_y, d,
            )
            .await?;

            Ok(())
        }

        "!chad" => {
            let settings =
                obs_filters::three_d_transform::ThreeDTransformPerspective {
                    scale_x: Some(217.0),
                    scale_y: Some(200.0),
                    rotation_x: Some(50.0),
                    field_of_view: Some(108.0),
                    camera_mode: (),
                    ..Default::default()
                };
            let d = models::DurationSettings {
                duration: Some(3000),
                ..Default::default()
            };
            let _ = move_transition::update_and_trigger_3d_filter(
                &obs_client,
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
            let arg_positions = default_move_or_scale_args();
            let req =
                build_chat_move_source_request(meat_of_message, &arg_positions);

            dbg!(&req);
            // Add Scale code
            Ok(())
        }

        "!alex" => {
            let source = "alex";
            let scene = "memes";
            let arg_positions = &default_move_or_scale_args()[1..];
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            let d = models::DurationSettings {
                duration: Some(req.duration as i32),
                easing_function_index: Some(req.easing_function_index),
                easing_type_index: Some(req.easing_type_index),
                ..Default::default()
            };

            move_transition::move_source_in_scene_x_and_y(
                &obs_client,
                scene,
                source,
                req.x,
                req.y,
                d,
            )
            .await
        }

        "!begin" => {
            let source = "begin";
            let scene = PRIMARY_CAM_SCENE;
            let arg_positions = &default_move_or_scale_args()[1..];
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            let d = models::DurationSettings {
                duration: Some(req.duration.try_into().unwrap_or(3000)),
                easing_function_index: Some(req.easing_function_index),
                easing_type_index: Some(req.easing_type_index),
                ..Default::default()
            };
            move_transition::move_source_in_scene_x_and_y(
                &obs_client,
                scene,
                source,
                req.x,
                req.y,
                d,
            )
            .await
        }

        // !move MEME_NAME X Y DURATION EASE-TYPE EASE-FUNCTION
        "!move" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = &default_move_or_scale_args();
            let req =
                build_chat_move_source_request(meat_of_message, arg_positions);

            let d = models::DurationSettings {
                duration: Some(req.duration.try_into().unwrap_or(3000)),
                easing_function_index: Some(req.easing_function_index),
                easing_type_index: Some(req.easing_type_index),
                ..Default::default()
            };
            move_transition::move_source_in_scene_x_and_y(
                &obs_client,
                &req.scene,
                &req.source,
                req.x,
                req.y,
                d,
            )
            .await
        }

        "!filter" => {
            println!("Trying Filter");
            // let default_filter_name = "Move_begin".to_string();
            let default_filter_name = "3D-Transform-Perspective".to_string();
            // "Move-3D-Transform-Orthographic".to_string();

            let filter: &str = splitmsg.get(1).unwrap_or(&default_filter_name);
            let filter_details =
                obs_client.filters().get("begin", filter).await?;

            println!("------------------------");
            println!("\n\tFilter Settings: {:?}", filter_details);
            println!("------------------------");
            Ok(())
        }

        "!twirl" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = &default_twirl_args();
            let req = build_chat_twirl_request(meat_of_message, arg_positions);
            let settings =
                obs_filters::three_d_transform::ThreeDTransformOrthographic {
                    rotation_y: Some(req.rotation_y),
                    camera_mode: (),
                    ..Default::default()
                };
            let d = models::DurationSettings {
                duration: Some(req.duration as i32),
                easing_function_index: None,
                easing_type_index: None,
                ..Default::default()
            };
            let _ = move_transition::update_and_trigger_3d_filter(
                &obs_client,
                source,
                "3D-Transform-Orthographic",
                settings,
                d,
            )
            .await;
            Ok(())
        }

        // Examples:
        //           !spin 1080 18000 ease-in-and-out cubic
        //
        // !spin SPIN_AMOUNT DURATION EASING-TYPE EASING-FUNCTION
        "!spin" | "!spinx" | "spiny" => {
            let arg_positions = &default_spin_args();
            let req = build_chat_move_source_request(
                splitmsg[1..].to_vec(),
                arg_positions,
            );

            let d = models::DurationSettings {
                duration: Some(req.duration as i32),
                easing_function_index: Some(req.easing_function_index),
                easing_type_index: Some(req.easing_type_index),
                ..Default::default()
            };
            move_transition::spin_source(
                &obs_client,
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
            _ = bootstrap::remove_all_filters(source, &obs_client).await;
            bootstrap::create_split_3d_transform_filters(source, &obs_client)
                .await
        }

        _ => Ok(()),
    };

    Ok(())
}

// Shoudl this be ChatArgPosition
fn default_wide_args() -> Vec<WideArgPosition> {
    vec![
        WideArgPosition::Source("begin".to_string()),
        WideArgPosition::X(500.0),
        WideArgPosition::Duration(3000),
    ]
}

fn default_spin_args() -> Vec<ChatArgPosition> {
    vec![
        ChatArgPosition::Source("begin".to_string()),
        ChatArgPosition::RotationZ(1080.0),
        ChatArgPosition::Duration(3000),
        ChatArgPosition::EasingType("ease-in-and-out".to_string()),
        ChatArgPosition::EasingFunction("sine".to_string()),
    ]
}

fn default_move_or_scale_args() -> Vec<ChatArgPosition> {
    vec![
        ChatArgPosition::Source("begin".to_string()),
        ChatArgPosition::X(1111.0),
        ChatArgPosition::Y(500.0),
        ChatArgPosition::Duration(3000),
        ChatArgPosition::EasingType("ease-in".to_string()),
        ChatArgPosition::EasingFunction("bounce".to_string()),
    ]
}

fn default_twirl_args() -> Vec<ChatArgPosition> {
    vec![
        ChatArgPosition::Source("begin".to_string()),
        ChatArgPosition::RotationY(360.0),
        ChatArgPosition::Duration(3000),
        ChatArgPosition::EasingType("ease-in".to_string()),
        ChatArgPosition::EasingFunction("bounce".to_string()),
    ]
}

#[derive(Default, Debug)]
pub struct TwirlRequest {
    pub source: String,
    pub rotation_y: f32,
    pub duration: u64,
    pub easing_type: String,
    pub easing_function: String,
    pub easing_type_index: i32,
    pub easing_function_index: i32,
}

fn build_chat_twirl_request(
    splitmsg: Vec<String>,
    arg_positions: &[ChatArgPosition],
) -> TwirlRequest {
    let mut req = TwirlRequest {
        ..Default::default()
    };
    for (index, arg) in arg_positions.iter().enumerate() {
        match arg {
            ChatArgPosition::Source(source) => {
                req.source = splitmsg.get(index).unwrap_or(source).to_string();
            }
            ChatArgPosition::RotationY(y) => {
                let str_y = format!("{}", y);
                req.rotation_y =
                    splitmsg.get(index).unwrap_or(&str_y).parse().unwrap_or(*y);
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
            _ => {
                // do nothing for values we don't care about
            }
        }
    }
    let (easing_type_index, easing_function_index) = find_easing_indicies(
        req.easing_type.clone(),
        req.easing_function.clone(),
    );

    req.easing_type_index = easing_type_index;
    req.easing_function_index = easing_function_index;
    return req;
}

pub fn build_chat_move_source_request(
    splitmsg: Vec<String>,
    arg_positions: &[ChatArgPosition],
) -> ChatMoveSourceRequest {
    let _default_source = "begin".to_string();
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
            _ => {}
        }
    }

    let (easing_type_index, easing_function_index) = find_easing_indicies(
        req.easing_type.clone(),
        req.easing_function.clone(),
    );

    req.easing_type_index = easing_type_index;
    req.easing_function_index = easing_function_index;

    let new_begin_source = constants::NEW_BEGIN_SOURCE;
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

pub fn build_wide_request(
    splitmsg: Vec<String>,
    arg_positions: &[WideArgPosition],
) -> Result<WideRequest> {
    let _default_source = "begin".to_string();
    let _default_scene = PRIMARY_CAM_SCENE.to_string();

    let mut req = WideRequest {
        ..Default::default()
    };

    for (index, arg) in arg_positions.iter().enumerate() {
        match arg {
            WideArgPosition::Source(source) => {
                req.source = splitmsg.get(index).unwrap_or(source).to_string()
            }
            WideArgPosition::X(_x) => {
                if let Some(x) = splitmsg
                    .get(index)
                    .and_then(|m| Some(m.parse::<f32>().unwrap_or(100.0)))
                {
                    req.x = x
                }
            }
            WideArgPosition::Duration(_duration) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs;

    #[tokio::test]
    async fn test_filters() {
        // let default_filter_name = "3D-Transform-Perspective".to_string();
        // klet default_filter_name = "3D-Transform-Orthographic".to_string();
        let default_filter_name = "Move_3D-Transform-Orthographic".to_string();

        let obs_client = obs::create_obs_client().await.unwrap();
        let filter_details = obs_client
            .filters()
            .get("begin", &default_filter_name)
            .await
            .unwrap();

        // let settings = ThreeDTransformPerspective {
        //     field_of_view: Some(122.6),
        //     camera_mode: (),
        //     ..Default::default()
        // };
        // let move_settings = MovePluginSettings {
        //     filter: default_filter_name
        //     settings,
        //     ..Default::default()
        // };

        // move_transition::spin_source(
        //     &req.source,
        //     req.rotation_z,
        //     3000,
        //     1,
        //     1,
        //     &obs_client,
        // )
        // .await;

        println!("------------------------");
        println!("\n\tFilter Settings: {:?}", filter_details);
        println!("------------------------");
    }
}
