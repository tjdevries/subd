use crate::skybox;
use crate::skybox::check_skybox_status_and_save;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use reqwest;
use rodio::*;
use serde;
use serde::{Deserialize, Serialize};
use serde_json;
use subd_types::Event;
use subd_types::UserMessage;
use tokio;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

#[allow(dead_code)]
pub struct Skybox {
    pub name: String,
    pub style_id: i32,
}

pub struct SkyboxHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
}

pub struct SkyboxRoutingHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub sink: Sink,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for SkyboxRoutingHandler {
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

            match handle_skybox_commands(
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

pub async fn handle_skybox_commands(
    tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    _pool: &sqlx::PgPool,
    _sink: &Sink,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<(), String> {
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();

    match command {
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
            return Ok(());
        }

        // This needs to take an ID
        "!skybox_styles" => {
            let styles = skybox::styles_for_chat().await;
            println!("\n\nStyles Time: {:?}", styles);

            // So we think this code isn't returning all chunks
            let chunks = chunk_string(&styles, 500);
            for chunk in chunks {
                println!("Chunk: {}", chunk);
                twitch_chat::client::send_message(twitch_client, chunk)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            return Ok(());
        }

        "!check_skybox" => {
            if not_beginbot {
                return Ok(());
            }

            // obs_client
            let _ = check_skybox_status_and_save(9612607).await;
            return Ok(());
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

            println!("Sending Skybox Request: {}", skybox_info.clone());
            let _ = tx.send(Event::SkyboxRequest(subd_types::SkyboxRequest {
                msg: skybox_info,
                style_id,
            }));

            return Ok(());
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
            return Ok(());
        }

        _ => {
            return Ok(());
        }
    };
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let request = match event {
                Event::SkyboxRequest(msg) => msg,
                _ => continue,
            };

            println!("Attempting to Skybox");
            match skybox::request_skybox(
                self.pool.clone(),
                request.msg,
                request.style_id,
            )
            .await
            {
                Ok(v) => {
                    // What
                }
                Err(e) => continue,
            };
        }
    }
}

// ============================================================================================
// ============================================================================================
// ============================================================================================
// ============================================================================================
//
// CHAT GPT Generated Code, BE CAREFUL

#[allow(dead_code)]
static SKYBOX_STATUS_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine/requests";
static SKYBOX_IMAGINE_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine";

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterRequest {
    pub response: Response,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkyboxStyle {
    pub id: i32,
    pub name: String,
    pub max_char: String,
    pub image: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemixRequestResponse {
    pub id: i32,
    pub obfuscated_id: String,
    pub user_id: i32,
    pub title: String,
    pub prompt: String,
    pub username: String,
    pub status: String,
    pub queue_position: i32,
    pub file_url: String,
    pub thumb_url: String,
    pub depth_map_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub error_message: Option<String>,
    pub pusher_channel: String,
    pub pusher_event: String,
    pub _type: String,
    pub skybox_style_id: i32,
    pub skybox_id: i32,
    pub skybox_style_name: String,
    pub skybox_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratorData {
    pub prompt: String,
    pub negative_text: String,
    pub animation_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub context: Option<String>,
    pub prompt: String,
    pub caption_text: Option<String>,
    pub author_name: String,
    pub alias_id: Option<String>,
    pub alias_name: Option<String>,
    pub progress: i32,
    pub status: String,
    pub queue_position: i32,
    pub file_url: String,
    pub thumb_url: String,
    pub video_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub media_version: i32,
    pub public: i32,
    pub error_message: Option<String>,
    pub _type: String,
    pub generator_data: GeneratorData,
    pub count_favorites: i32,
    pub likes_count: i32,
    pub user_imaginarium_image_left: i32,
}

// Why are we passing the API Key in the URL?
#[allow(dead_code)]
async fn request_status(id: &str) -> Result<Response> {
    let skybox_api_key: String = std::env::var("SKYBOX_API_KEY").unwrap();
    let url = format!(
        "{}/requests/{}?api_key={}",
        SKYBOX_IMAGINE_URL, id, skybox_api_key
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;

    let parsed_response: OuterRequest = serde_json::from_str(&body)?;

    Ok(parsed_response.response)
}

// async fn remix(remix_id: i32, style_id: i32, prompt: &str) -> Result<String, Box<dyn Error>> {
//     // Perform HTTP POST request here...
//     let requests_url = format!("{}?api_key={}", SKYBOX_REMIX_URL, SKYBOX_API_KEY);
//     // Generate the post body and perform the HTTP request...
//
//     let response_body = reqwest::Client::new().post(&requests_url).json(&map).send().await?;
//     let body = response_body.text().await?;
//
//     // Write to file here...
//
//     let skybox_remix_response_file_path = "/home/begin/code/subd/tmp/skybox_archive";
//     Ok(skybox_remix_response_file_path)
// }

pub fn find_style_id(splitmsg: Vec<String>) -> i32 {
    println!("\t Splitmsg: {:?}", splitmsg);
    // TODO: Do a search on Blockade ID for the values
    let range = 1..=47;
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

// pub fn build_chat_move_source_request(
//     splitmsg: Vec<String>,
//     arg_positions: Vec<ChatArgPosition>,
// ) -> ChatMoveSourceRequest {
//     let default_source = "begin".to_string();
//     let default_scene = PRIMARY_CAM_SCENE.to_string();
//
//     let mut req = ChatMoveSourceRequest {
//         ..Default::default()
//     };
//
//     for (index, arg) in arg_positions.iter().enumerate() {
//         match arg {
//             ChatArgPosition::Source(source) => {
//                 req.source = splitmsg.get(index).unwrap_or(source).to_string();
//             }
//             ChatArgPosition::RotationZ(z) => {
//                 let str_z = format!("{}", z);
//                 req.rotation_z =
//                     splitmsg.get(index).unwrap_or(&str_z).parse().unwrap_or(*z);
//             }
//             ChatArgPosition::X(x) => {
//                 let str_x = format!("{}", x);
//                 req.x =
//                     splitmsg.get(index).unwrap_or(&str_x).parse().unwrap_or(*x);
//             }
//             ChatArgPosition::Y(y) => {
//                 let str_y = format!("{}", y);
//                 req.y = splitmsg
//                     .get(index)
//                     .unwrap_or(&str_y)
//                     .to_string()
//                     .parse()
//                     .unwrap_or(*y);
//             }
//             ChatArgPosition::Duration(duration) => {
//                 let str_duration = format!("{}", duration);
//                 req.duration = splitmsg
//                     .get(index)
//                     .unwrap_or(&str_duration)
//                     .to_string()
//                     .parse()
//                     .unwrap_or(*duration);
//             }
//             ChatArgPosition::EasingType(easing_type) => {
//                 req.easing_type =
//                     splitmsg.get(index).unwrap_or(easing_type).to_string()
//             }
//             ChatArgPosition::EasingFunction(easing_function) => {
//                 req.easing_function =
//                     splitmsg.get(index).unwrap_or(easing_function).to_string()
//             }
//         }
//     }
//
//     return req;
// }

// TODO: We need to rneamr all of these
pub enum ChatArgPosition {
    Source(String),
    X(f32),
    Y(f32),
    RotationZ(f32),
    Duration(u64),
    EasingType(String),
    EasingFunction(String),
}
