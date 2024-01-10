use crate::bootstrap;
use crate::move_transition;
use crate::move_transition_effects;
use crate::obs;
use crate::obs_scenes;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws;
use obws::Client as OBSClient;
use rodio::*;
use std::collections::HashMap;
use subd_twitch::rewards;
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
    _pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let default_source = obs::DEFAULT_SOURCE.to_string();
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
    let _scene = obs_scenes::find_scene(source)
        .await
        .unwrap_or(obs::MEME_SCENE.to_string());
    let command = splitmsg[0].as_str();

    let _ = match command {
        // !wide SOURCE WIDTH DURATION
        "!wide" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = default_wide_args();
            let req = build_wide_request(meat_of_message, &arg_positions)?;
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

        // ===========================================
        // == Scaling Sources
        // ===========================================
        "!grow" | "!scale" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = default_move_or_scale_args();
            let req =
                build_chat_move_source_request(meat_of_message, &arg_positions);

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

        "!alex" => {
            let source = "alex";
            let scene = "memes";
            let arg_positions = &default_move_or_scale_args()[1..];
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
            let arg_positions = &default_move_or_scale_args()[1..];
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

        // !move MEME_NAME X Y DURATION EASE-TYPE EASE-FUNCTION
        "!move" => {
            let meat_of_message = splitmsg[1..].to_vec();
            let arg_positions = &default_move_or_scale_args();
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

        // This sets up OBS for Begin's current setup
        "!create_filters_for_source" => {
            if _not_beginbot {
                return Ok(());
            }
            let default = "alex".to_string();
            let source: &str = splitmsg.get(1).unwrap_or(&default);
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
        ChatArgPosition::RotationZ(1090.0),
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

pub fn build_chat_move_source_request(
    splitmsg: Vec<String>,
    arg_positions: &[ChatArgPosition],
) -> move_transition::ChatMoveSourceRequest {
    let _default_source = "begin".to_string();
    let default_scene = PRIMARY_CAM_SCENE.to_string();

    let mut req = move_transition::ChatMoveSourceRequest {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filters() {
        // let default_filter_name = "3D-Transform-Perspective".to_string();
        let default_filter_name = "3D-Transform-Orthographic".to_string();
        // "Move-3D-Transform-Orthographic".to_string();

        let obs_client = obs::create_obs_client().await.unwrap();
        let filter_details = obs_client
            .filters()
            .get("begin", &default_filter_name)
            .await
            .unwrap();

        println!("------------------------");
        println!("\n\tFilter Settings: {:?}", filter_details);
        println!("------------------------");
    }
}
