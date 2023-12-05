use crate::music_scenes;
use crate::obs_scenes;
use crate::openai;
use crate::redemptions;
use anyhow::Result;
use async_trait::async_trait;
use axum::routing::post;
use axum::{
    http::StatusCode, response::IntoResponse, Extension, Json, Router, Server,
};
use events::EventHandler;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use subd_types::Event;
use tokio::sync::broadcast;
use twitch_chat::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScenes {
    pub scenes: Vec<AIScene>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScene {
    pub reward_title: String,
    pub base_prompt: String,
    pub base_dalle_prompt: String,
    pub voice: String,
    pub music_bg: String,
}

pub struct TwitchEventSubHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventSubRoot {
    pub subscription: Subscription,
    pub event: Option<SubEvent>,
    pub challenge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    id: String,
    status: String,
    #[serde(rename = "type")]
    type_field: String,
    version: String,
    condition: Condition,
    // condition: HashMap<String, String>,
    transport: Transport,
    created_at: String,
    // cost: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Condition {
    broadcaster_user_id: String,
    reward_id: String,
    // Will this crash shit
    // user_input: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Reward {
    title: String,
    cost: i32,
    id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubEvent {
    id: Uuid,
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    title: Option<String>,
    tier: Option<String>,
    is_gift: Option<bool>,
    reward: Option<Reward>,
    user_input: Option<String>,
}

#[async_trait]
impl EventHandler for TwitchEventSubHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let clonable_obs_client = Arc::new(self.obs_client);
        let clonable_pool = Arc::new(self.pool);

        // Define the route
        let app = Router::new()
            .route("/eventsub", post(post_request))
            .layer(Extension(clonable_obs_client))
            .layer(Extension(clonable_pool))
            .layer(Extension(tx))
            .layer(Extension(self.twitch_client));

        // Run the Axum server in a separate async task
        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
            Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        Ok(())
    }
}

async fn post_request(
    Json(eventsub_body): Json<EventSubRoot>,
    Extension(obs_client): Extension<Arc<OBSClient>>,
    Extension(pool): Extension<Arc<sqlx::PgPool>>,
    Extension(tx): Extension<broadcast::Sender<Event>>,
    Extension(twitch_client): Extension<
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    >,
) -> impl IntoResponse {
    // We could check our DB first, before printing this
    // We want this to occur later on, after some filtering
    // dbg!(&eventsub_body);

    // We need to read in the json file
    let file_path = "/home/begin/code/subd/data/AIScenes.json";
    let contents = fs::read_to_string(file_path).expect("Can read file");

    let ai_scenes: AIScenes = serde_json::from_str(&contents).unwrap();

    let ai_scenes_map: HashMap<String, &AIScene> = ai_scenes
        .scenes
        .iter()
        .map(|scene| (scene.reward_title.clone(), scene))
        .collect();

    match eventsub_body.challenge {
        Some(challenge) => {
            println!("We got a challenge!");
            // This is required for EventSub's to work!
            // If we don't Twitch's challenge, you don't events
            return (StatusCode::OK, challenge);
        }
        _ => {}
    }

    let c = obs_client;

    match eventsub_body.subscription.type_field.as_str() {
        // What if we checked for Polls here!
        "channel.follow" => {
            println!("follow time");
        }

        // I don't know if the eventsub_body will match
        "channel.poll.begin" => {
            println!("\nPOLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL");
        }
        "channel.poll.progress" => {
            println!("\nPOLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL");
        }
        "channel.poll.end" => {
            println!("\nPOLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL");
        }

        "channel.channel_points_custom_reward_redemption.add" => {
            match eventsub_body.event {
                Some(event) => match event.reward {
                    Some(reward) => {
                        let command = reward.title.clone();
                        let user_input = match event.user_input.clone() {
                            Some(input) => input,
                            None => "".to_string(),
                        };

                        // Should we return error status code?
                        if user_input == "".to_string() {
                            println!("No user input");
                            return (StatusCode::OK, "".to_string());
                        };

                        let old_redemp =
                            redemptions::find_redemption_by_reward_id(
                                &pool,
                                event.id.clone(),
                            )
                            .await;
                        match old_redemp {
                            Ok(_reward_id) => {
                                println!(
                                    "\nWe found a redemption: {}\n",
                                    command.clone()
                                );
                                return (StatusCode::OK, "".to_string());
                            }
                            Err(e) => {
                                println!("\nNo redemption found, saving new redemption: {:?} | Command: {} ID: {}\n", e, command.clone(), event.id.clone());

                                let _ = redemptions::save_redemptions(
                                    &pool,
                                    command.clone(),
                                    reward.cost.clone(),
                                    event.user_name.clone(),
                                    event.id.clone(),
                                    user_input,
                                )
                                .await;
                            }
                        }

                        match ai_scenes_map.get(&command) {
                            Some(scene) => {
                                let user_input = event.user_input.unwrap();

                                // let thang = "dogs".to_string();
                                // let chat_response = openai::ask_chat_gpt("Description the following".to_string(), thang).await;

                                let base_prompt = scene.base_prompt.clone();
                                println!(
                                    "Asking Chat GPT: {} - {}",
                                    base_prompt, user_input
                                );
                                let chat_response = openai::ask_chat_gpt(
                                    user_input.clone().to_string(),
                                    base_prompt,
                                )
                                .await;
                                // let content = chat_response.unwrap().content.unwrap().to_string();

                                // What the heck is going wrong here?
                                // Is there are eerrors we should jsut return
                                let content = match chat_response {
                                    Ok(response) => {
                                        match response.content {
                                            Some(c) => c,
                                            None => {
                                                // Handle the case where content is None
                                                // You can return a default value or handle the error as needed
                                                "Default content".to_string() // Example default value
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        // Handle the error case of chat_response
                                        // Log the error, return a default value, or perform other error handling
                                        eprintln!("Error occurred: {:?}", e); // Example error logging
                                        "Error response".to_string() // Example default value
                                    }
                                };
                                println!(
                                    "Chat GPT response: {:?}",
                                    content.clone()
                                );

                                let dalle_mode = true;
                                let dalle_prompt = if dalle_mode {
                                    let base_dalle_prompt =
                                        scene.base_dalle_prompt.clone();
                                    let dalle_response = openai::ask_chat_gpt(
                                        user_input.clone(),
                                        base_dalle_prompt,
                                    )
                                    .await;
                                    let dalle_content = dalle_response
                                        .unwrap()
                                        .content
                                        .unwrap()
                                        .to_string();
                                    Some(dalle_content)
                                } else {
                                    None
                                };
                                println!(
                                    "Dalle GPT response: {:?}",
                                    dalle_prompt.clone()
                                );

                                let _ = trigger_full_scene(
                                    tx.clone(),
                                    scene.voice.clone(),
                                    scene.music_bg.clone(),
                                    content,
                                    dalle_prompt,
                                )
                                .await;
                            }
                            None => {
                                println!("Scene not found for reward title")
                            }
                        }

                        // let ai_scene = ai_scene_map[command.clone().to_string()];
                        match command.clone().as_str() {
                            "gallery" => {
                                let _ =
                                    obs_scenes::change_scene(&c, "art_gallery")
                                        .await;
                            }
                            "code" => {
                                let _ = obs_scenes::change_scene(&c, "Primary")
                                    .await;
                            }

                            _ => {
                                for &(cmd, ref _scene) in
                                    music_scenes::VOICE_TO_MUSIC.iter()
                                {
                                    let cmd_no_bang = &cmd[1..];

                                    if cmd_no_bang == command.clone() {
                                        let _ = send_message(
                                            &twitch_client,
                                            format!("!{}", command.clone()),
                                        )
                                        .await;
                                    }
                                }
                            }
                        };
                    }
                    None => {
                        println!("NO REWARD FOUND!")
                    }
                },
                None => {
                    println!("NO EVENT FOUND!")
                }
            }
        }
        _ => {
            println!("NO EVENT FOUND!")
        }
    };

    (StatusCode::OK, "".to_string())
}

async fn trigger_full_scene(
    tx: broadcast::Sender<Event>,
    voice: String,
    music_bg: String,
    content: String,
    dalle_prompt: Option<String>,
) -> Result<()> {
    match dalle_prompt {
        Some(prompt) => {
            println!("\n\tDalle Prompt: {}", prompt.clone().to_string());
            let _ =
                tx.send(Event::AiScenesRequest(subd_types::AiScenesRequest {
                    voice: Some(voice),
                    message: content.clone(),
                    voice_text: content.clone(),
                    music_bg: Some(music_bg),
                    dalle_prompt: Some(prompt),
                    ..Default::default()
                }));
        }
        None => {
            println!("\n\tDalle Prompt: None");
            let _ =
                tx.send(Event::AiScenesRequest(subd_types::AiScenesRequest {
                    voice: Some(voice),
                    message: content.clone(),
                    voice_text: content,
                    music_bg: Some(music_bg),
                    dalle_prompt: None,
                    ..Default::default()
                }));
        }
    }
    Ok(())
}
