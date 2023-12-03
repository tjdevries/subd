use crate::music_scenes;
use crate::obs_scenes;
use anyhow::Result;
use async_trait::async_trait;
use axum::routing::post;
use axum::{
    http::StatusCode, response::IntoResponse, Extension, Json, Router, Server,
};
use dotenv::dotenv;
use events::EventHandler;
use obws::Client as OBSClient;
use openai::chat::{ChatCompletion, ChatCompletionDelta};
use openai::{
    chat::{ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{
    env,
    io::{stdin, stdout, Write},
};
use subd_types::Event;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;
use twitch_chat::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct TwitchEventSubHandler {
    pub obs_client: OBSClient,
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
    cost: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Condition {
    broadcaster_user_id: String,
    reward_id: String,
    // Will this crash shit
    // user_input: Option<String>,
}
// "reward": {
//         "id": "92af127c-7326-4483-a52b-b0da0be61c01",
//         "title": "title",
//         "cost": 100,
//         "prompt": "reward prompt"
//     },

#[derive(Serialize, Deserialize, Debug)]
struct Reward {
    title: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubEvent {
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

        // Define the route
        let app = Router::new()
            .route("/eventsub", post(post_request))
            .layer(Extension(clonable_obs_client))
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
    Extension(tx): Extension<broadcast::Sender<Event>>,
    Extension(twitch_client): Extension<
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    >,
) -> impl IntoResponse {
    println!("\t~~ Eventsub Body: {:?}", eventsub_body);
    let voice_dir = "/home/begin/code/BeginGPT/tmp/current";

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
                        let command = reward.title.as_ref();
                        match command {
                                "Hospital Commercial" => {
                                println!("Time to ask Chat GPT");
                                let user_input = event.user_input.unwrap();
                                
                                let base_content =  "Don't include directions, or instructions. Just the words a voice-over would contain. Be Short. Be Concise. Don't mention the inspiration. Never say more than 80 words. Act like a Hospital or pharmaceutical comerical.".to_string();

                                let chat_response = ask_chat_gpt(
                                    user_input, base_content,
                                )
                                .await;

                                // This unwrap might fail
                                let content =
                                    chat_response.content.unwrap().to_string();

                                let voice = "bella".to_string();
                                let _ = tx.send(Event::ElevenLabsRequest(
                                    subd_types::ElevenLabsRequest {
                                        voice: Some(voice),
                                        message: content.clone(),
                                        voice_text: content,
                                        music_bg: Some("!hospital".to_string()),
                                        ..Default::default()
                                    },
                                ));
                            }
                            
                            "Ask Begin Jones a Question" => {
                                println!("Time to ask Chat GPT");
                                let user_input = event.user_input.unwrap();
                                
                                let base_content =  "You're a conspiracy theorist, you give us wild theories on what we ask. But never more than 80 words".to_string();
                                let chat_response = ask_chat_gpt(
                                    user_input, base_content,
                                )
                                .await;

                                // This unwrap might fail
                                let content =
                                    chat_response.content.unwrap().to_string();

                                let voice = "ethan".to_string();
                                let _ = tx.send(Event::ElevenLabsRequest(
                                    subd_types::ElevenLabsRequest {
                                        voice: Some(voice),
                                        message: content.clone(),
                                        voice_text: content,
                                        music_bg: Some("!drama".to_string()),
                                        ..Default::default()
                                    },
                                ));
                            }

                            "gallery" => {
                                let _ = obs_scenes::change_scene(&c, "4 Piece")
                                    .await;
                            }

                            "code" => {
                                let _ = obs_scenes::change_scene(&c, "Primary")
                                    .await;
                            }

                            _ => {
                                for &(cmd, ref scene) in
                                    music_scenes::VOICE_TO_MUSIC.iter()
                                {
                                    println!(
                                        "Reward Title {} - Music: {:?}",
                                        reward.title, scene.music
                                    );

                                    let cmd_no_bang = &cmd[1..];

                                    if cmd_no_bang == reward.title {
                                        let _ = send_message(
                                            &twitch_client,
                                            format!("!{}", reward.title),
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
async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> ChatCompletionMessage {
    set_key(env::var("OPENAI_KEY").unwrap());

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(base_content),
        name: None,
        function_call: None,
    }];

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(user_input),
        name: None,
        function_call: None,
    });

    let chat_completion =
        ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .create()
            .await
            .unwrap();
    let returned_message =
        chat_completion.choices.first().unwrap().message.clone();
    println!(
        "Returned Messages {:#?}: {}",
        &returned_message.role,
        &returned_message.content.clone().unwrap().trim()
    );
    returned_message
}
