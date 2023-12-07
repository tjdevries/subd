use crate::openai;
use crate::redemptions;
use crate::twitch_stream_state;
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
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use subd_types::Event;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::time::timeout;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
// use crate::music_scenes;
// use crate::obs_scenes;
// use std::env;
// use twitch_chat::send_message;

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
    dbg!(&eventsub_body);

    // We need to read in the json file
    let file_path = "/home/begin/code/subd/data/AIScenes.json";
    let contents = fs::read_to_string(file_path).expect("Can read file");

    let ai_scenes: AIScenes = serde_json::from_str(&contents).unwrap();

    let ai_scenes_map: HashMap<String, &AIScene> = ai_scenes
        .scenes
        .iter()
        .map(|scene| (scene.reward_title.clone(), scene))
        .collect();

    // This is required for EventSub's to work!
    // If we don't Twitch's challenge, you don't events
    match eventsub_body.challenge {
        Some(challenge) => {
            println!("We got a challenge!");
            return (StatusCode::OK, challenge);
        }
        _ => {}
    }

    match eventsub_body.subscription.type_field.as_str() {
        "channel.follow" => {
            println!("follow time");
        }
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
                Some(event) => {
                    let _ =
                        handle_ai_scene(tx, pool, ai_scenes_map, event).await;
                }
                None => {
                    println!("NO Event Found for redemption!")
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

async fn find_or_save_redemption(
    pool: Arc<sqlx::PgPool>,
    id: Uuid,
    command: String,
    reward_cost: i32,
    user_name: String,
    user_input: String,
) -> Result<()> {
    let old_redemp = redemptions::find_redemption_by_reward_id(&pool, id).await;

    match old_redemp {
        Ok(_reward_id) => {
            println!("\nWe found a redemption: {}\n", command.clone());
            return Ok(());
        }
        Err(e) => {
            println!("\nNo redemption found, saving new redemption: {:?} | Command: {} ID: {}\n", e, command.clone(), id.clone());

            let _ = redemptions::save_redemptions(
                &pool,
                command,
                reward_cost.clone(),
                user_name,
                id,
                user_input,
            )
            .await;
        }
    }
    Ok(())
}

async fn handle_ai_scene(
    tx: broadcast::Sender<Event>,
    pool: Arc<sqlx::PgPool>,
    ai_scenes_map: HashMap<String, &AIScene>,
    event: SubEvent,
) -> Result<()> {
    let state = twitch_stream_state::get_twitch_state(&pool).await?;
    let dalle_mode = state.dalle_mode;

    let reward = event.reward.unwrap();
    let command = reward.title.clone();

    let user_input = match event.user_input.clone() {
        Some(input) => input,
        None => return Ok(()),
    };

    let _ = find_or_save_redemption(
        pool.clone(),
        event.id.clone(),
        command.clone(),
        reward.cost.clone(),
        event.user_name.clone(),
        user_input.clone(),
    )
    .await;

    match ai_scenes_map.get(&command) {
        Some(scene) => {
            let user_input = event.user_input.unwrap();
            let base_prompt = scene.base_prompt.clone();
            println!("Asking Chat GPT: {} - {}", base_prompt, user_input);

            let chat_response = openai::ask_chat_gpt(
                user_input.clone().to_string(),
                base_prompt,
            )
            .await;

            // let content = chat_response.unwrap().content.unwrap().to_string();
            let content = match chat_response {
                Ok(response) => response.content.unwrap(),
                Err(e) => {
                    eprintln!("Error occurred: {:?}", e); // Example error logging
                    "Error response".to_string() // Example default value
                }
            };
            println!("Chat GPT response: {:?}", content.clone());

            let dalle_prompt = if dalle_mode {
                let base_dalle_prompt = scene.base_dalle_prompt.clone();

                // This is not using Chat-GPT
                let prompt = format!("{} {}", base_dalle_prompt, user_input);
                Some(prompt)

                // This is Danvinci
                // let dalle_response = openai::ask_davinci(
                //     user_input.clone(),
                //     base_dalle_prompt.clone(),
                // )
                // .await;

                // This is Chat GPT
                // let dalle_response = openai::ask_chat_gpt(
                //     user_input.clone(),
                //     base_dalle_prompt.clone(),
                // )
                // .await;
                // match dalle_response {
                //     Ok(chat_completion) => {
                //         Some(chat_completion.content.unwrap())
                //     },
                //     Err(e) => {
                //         eprintln!("Error finding Dalle Content: {:?}", e);
                //         None
                //     }
                // }
            } else {
                None
            };

            println!("Dalle GPT response: {:?}", dalle_prompt.clone());

            // New Theory:
            //             // Calling this trigger_full_scene, stops it from printing Dalle GPT
            //                response above
            let _ = trigger_full_scene(
                tx.clone(),
                scene.voice.clone(),
                scene.music_bg.clone(),
                content.clone(),
                dalle_prompt.clone(),
            )
            .await;
        }
        None => {
            println!("Scene not found for reward title")
        }
    }

    Ok(())
}
