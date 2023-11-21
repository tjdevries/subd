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

// {"subscription":{"id":"ddfd7140-1590-1dda-ca56-61aac70f9be1","status":"enabled","type":"channel.follow","version":"1","condition":{"broadcaster_user_id":"10483564"},"transport":{"method":"webhook","callback":"null"},"created_at":"2023-11-20T22:57:37.179519123Z","cost":0},"event":{"user_id":"25578051","user_login":"testFromUser","user_name":"testFromUser","broadcaster_user_id":"10483564","broadcaster_user_login":"10483564","broadcaster_user_name":"testBroadcaster","followed_at":"2023-11-20T22:57:37.179519123Z"}}
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
    condition: HashMap<String, String>,
    transport: Transport,
    created_at: String,
    cost: i32,
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
    tier: Option<String>,
    is_gift: Option<bool>,
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
                // This is required for EventSub's to work!
                // If we don't Twitch's challenge, you don't events
                challenge
        }
        _ =>  {

            let c = obs_client;

           let _ = send_message(&twitch_client, "I assure we are quite operational.").await;
            
            let _ = obs_source::set_enabled(
                "Primary",
                "Dalle-Gen-1",
                true,
                &c,
            ).await;
            
            let _ = tx.send(Event::UberDuckRequest(subd_types::UberDuckRequest {
                    message: "woah there budy".to_string(),
                    voice_text: "waoh there buddy".to_string(),
                    voice: "Ethan".to_string(),
                    username: "beginbot".to_string(),
                    source: None,
            }));
            

            "".to_string()
        }
    };

    (StatusCode::OK, challenge)
}
