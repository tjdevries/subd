use anyhow::Result;
use std::sync::Mutex;
use std::sync::MutexGuard;
use twitch_chat::send_message;
use std::sync::Arc;
use events::EventHandler;
use tokio::sync::broadcast;
use subd_types::Event;
use obws::Client as OBSClient;
use async_trait::async_trait;
use std::net::SocketAddr;
use crate::obs;
use crate::obs_source;
use crate::obs_scenes;
use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use axum::{
    routing::{get, post},
    Router, Server, Extension, Json,
    http::StatusCode,
    response::IntoResponse,
};
use twitch_irc::{TwitchIRCClient, SecureTCPTransport, login::StaticLoginCredentials};

pub struct TwitchEventSubHandler {
    pub obs_client: OBSClient,
    pub twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
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
struct Condition{
    broadcaster_user_id: String,
    reward_id: String,
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
}

#[async_trait]
impl EventHandler for TwitchEventSubHandler {
    
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
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
            Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
        });

        Ok(())
    }
}

async fn post_request(
    Json(eventsub_body): Json<EventSubRoot>,
    Extension(obs_client): Extension<Arc<OBSClient>>,
    Extension(tx): Extension<broadcast::Sender<Event>>,
    Extension(twitch_client): Extension<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
) -> impl IntoResponse {

    println!("simple_map = {:?}", eventsub_body);
    
    let challenge = match eventsub_body.challenge {
        Some(challenge) => {
                println!("We got a challenge!");
                // This is required for EventSub's to work!
                // If we don't Twitch's challenge, you don't events
                challenge
        }
        _ =>  {
            let c = obs_client;
            match eventsub_body.subscription.type_field.as_str() {
                "channel.follow" => {
                    println!("follow time");
                },
                "channel.channel_points_custom_reward_redemption.add" => {
                    match eventsub_body.event {
                    Some(event) => {
                        match event.reward {
                            Some(reward) => {
                                    println!("REWARD TITLE: {}", reward.title);

                                    match reward.title.as_ref() {
                                        "gallery" => {
                                            let _ = obs_scenes::change_scene(&c, "4 Piece").await;
                                        },
                                        "code" => {
                                            let _ = obs_scenes::change_scene(&c, "Primary").await;
                                        },
                                            
                                        // I should be reading this from a Map, like the one in
                                        // obs_routing.rs
                                        "ken" | "drama" | "yoga" | "news" | "romcom" | "sigma" | "hospital" | "greed" | "sexy" | "chef"  => {
                                            let _ = send_message(&twitch_client, format!("!{}", reward.title)).await;
                                            // let _ = send_message(&twitch_client, "!ken").await;
                                        },
                                        _ => {
                                                println!("Couldn't find reward in options")
                                        }
                                    };
                            }
                            None => {println!("No reward found!")}
                        }
                        }
                        None => {println!("No event found!")}
                    }
                    
                },
                _ => println!("nothing"),
            };
            
           // let _ = send_message(&twitch_client, "I assure we are quite operational.").await;
           //  
           // let _ = obs_source::set_enabled(
           //      "Primary",
           //      "Dalle-Gen-1",
           //      true,
           //      &c,
           //  ).await;
           //  
           // let _ = tx.send(Event::UberDuckRequest(subd_types::UberDuckRequest {
           //          message: "woah there budy".to_string(),
           //          voice_text: "waoh there buddy".to_string(),
           //          voice: "Ethan".to_string(),
           //          username: "beginbot".to_string(),
           //          source: None,
           //  }));
            

            "".to_string()
        }
    };

    (StatusCode::OK, challenge)
}
