use anyhow::Result;
use obs_move_transition::duration::find_easing_indicies;

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
    pub source: String,
    pub _scene: String,
    pub x: f32,
    pub duration: u64,
}

// Shoudl this be ChatArgPosition
pub fn default_wide_args() -> Vec<WideArgPosition> {
    vec![
        WideArgPosition::Source("begin".to_string()),
        WideArgPosition::X(500.0),
        WideArgPosition::Duration(3000),
    ]
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

pub fn default_spin_args() -> Vec<ChatArgPosition> {
    vec![
        ChatArgPosition::Source("begin".to_string()),
        ChatArgPosition::RotationZ(1080.0),
        ChatArgPosition::Duration(3000),
        ChatArgPosition::EasingType("ease-in-and-out".to_string()),
        ChatArgPosition::EasingFunction("sine".to_string()),
    ]
}

pub fn default_move_or_scale_args() -> Vec<ChatArgPosition> {
    vec![
        ChatArgPosition::Source("begin".to_string()),
        ChatArgPosition::X(1111.0),
        ChatArgPosition::Y(500.0),
        ChatArgPosition::Duration(3000),
        ChatArgPosition::EasingType("ease-in".to_string()),
        ChatArgPosition::EasingFunction("bounce".to_string()),
    ]
}

pub fn default_twirl_args() -> Vec<ChatArgPosition> {
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

pub fn build_chat_twirl_request(
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

// Parsing Chat
// This is related to parsing chat
pub fn build_chat_move_source_request(
    splitmsg: Vec<String>,
    arg_positions: &[ChatArgPosition],
) -> ChatMoveSourceRequest {
    let _default_source = "begin".to_string();
    let default_scene =
        subd_types::consts::get_primary_camera_scene().to_string();

    let mut req = ChatMoveSourceRequest::default();

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

    let new_begin_source = subd_types::consts::get_ai_twin_obs_source();
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

pub fn build_wide_request(
    splitmsg: Vec<String>,
    arg_positions: &[WideArgPosition],
) -> Result<WideRequest> {
    let _default_source = "begin".to_string();

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
    // use super::*;
    use obs_service::obs;

    #[tokio::test]
    async fn test_filters() {
        // let default_filter_name = "3D-Transform-Perspective".to_string();
        // klet default_filter_name = "3D-Transform-Orthographic".to_string();
        let default_filter_name = "3D-Transform-Orthographic".to_string();

        println!("Am I losing it?");
        let obs_client = obs::create_obs_client().await.unwrap();
        let filter_details = obs_client
            .filters()
            .get("begin", &default_filter_name)
            .await
            .unwrap();
        println!("------------------------");
        println!("\n\tFilter Settings: {:?}", filter_details);
        println!("------------------------");

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
    }
}
