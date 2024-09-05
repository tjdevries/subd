use crate::ai_scenes;
use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::move_transition_effects;
use crate::constants;
use crate::obs_combo;
use crate::obs::obs;
use crate::obs::obs_hotkeys;
use crate::obs::obs_scenes;
use crate::obs::obs_source;
use crate::openai::openai;
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
    let default_source = constants::DEFAULT_SOURCE.to_string();
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let source: &str = splitmsg.get(1).unwrap_or(&default_source);
    let duration: u32 = splitmsg
        .get(4)
        .map_or(3000, |x| x.trim().parse().unwrap_or(3000));
    let filter_value = splitmsg
        .get(3)
        .map_or(0.0, |x| x.trim().parse().unwrap_or(0.0));
    let scene = match obs_scenes::find_scene(source).await {
        Ok(scene) => scene.to_string(),
        Err(_) => constants::MEME_SCENE.to_string(),
    };

    let command = splitmsg[0].as_str();

    let _ = match command {
        "!flash_sale" => {
            if not_beginbot {
                return Ok(());
            }
            let broadcaster_id = "424038378";

            let file_path = "/home/begin/code/subd/data/AIScenes.json";
            let contents =
                fs::read_to_string(file_path).expect("Can read file");
            let ai_scenes: ai_scenes::AIScenes =
                serde_json::from_str(&contents.clone())
                    .map_err(|e| e.to_string())?;

            // Why aren't we passing this in?
            // This is need to create Reward Manager
            // TODO: Hook this up to regenrating on expirartion
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
            let reward_res =
                twitch_rewards::find_by_title(&pool, title.to_string())
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
                println!("Scene: {:?}", scene);
                
                if scene.reward_title == "Ask Melkey a Question" {
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

            Ok(())
        }

        "!set_character" => Ok(()),

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

    if req.source == default_source {
        req.scene = default_scene;
    } else {
        req.scene = "Memes".to_string();
    };

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
        let obs_client = constants::create_obs_client().await.unwrap();

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
