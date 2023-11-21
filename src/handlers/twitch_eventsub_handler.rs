use anyhow::Result;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::Arc;
use events::EventHandler;
use tokio::sync::broadcast;
use subd_types::Event;
use obws::Client as OBSClient;
use async_trait::async_trait;

use crate::obs;
use crate::obs_source;
use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use warp::{http::StatusCode, Filter, Rejection, Reply, reply, reply::json, Buf};


pub struct TwitchEventSubHandler {
    pub obs_client: OBSClient,
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


// 60 | fn with_db(obs_client: Arc<Mutex<OBSClient>>) -> impl Filter<Extract = (Arc<Mutex<OBSClient>>,), Error = std::convert::Infallible> {
//    |                                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `MutexGuard<'_, Client>`, found `Arc<Mutex<Client>>`
// fn with_db(obs_client: Arc<Mutex<OBSClient>>) -> impl Filter<Extract = (Arc<Mutex<OBSClient>>,), Error = std::convert::Infallible> {
// 
// fn with_db(obs_client: Arc<Mutex<OBSClient>>) -> impl Filter<Extract = (MutexGuard<'_, OBSClient>,), Error = std::convert::Infallible> {
//     warp::any().map(move || obs_client.lock().unwrap())
//     // warp::any().map(move || obs_client)
// }
    

#[async_trait]
impl EventHandler for TwitchEventSubHandler {
    
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        
        let clonable_obs_client = Arc::new(Mutex::new(self.obs_client));
        
        // How do I pass more things to Warp?
        let route = warp::body::content_length_limit(1024 * 1282)
            .and(warp::body::json())
            .and_then(post_request);
        
        // Run the Warp server in a separate async task
        tokio::spawn(async move {
            warp::serve(route).run(([0, 0, 0, 0], 8080)).await;
        });

        loop {
            let event = rx.recv().await?;
            // TODO: Update this fren
            let msg = match event {
                Event::StreamCharacterRequest(msg) => msg,
                _ => continue,
            };

        }
    }
}

async fn post_request(eventsub_body: EventSubRoot) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    println!("simple_map = {:?}", eventsub_body);
    
    let challenge = match eventsub_body.challenge {
        Some(challenge) => {
                // This is required for EventSub's to work!
                // If we don't Twitch's challenge, you don't events
                challenge
        }
        _ =>  {

            // let obs_client = obs::create_obs_client().await.unwrap();
            // obs_source::set_enabled(
            //     "Primary",
            //     "Dalle-Gen-1",
            //     true,
            //     &self.obs_client,
            // ).await;
            
            // let _ = self.tx.send(Event::UberDuckRequest(subd_types::UberDuckRequest {
            //         message: "woah there budy".to_string(),
            //         voice_text: "waoh there buddy".to_string(),
            //         voice: "Ethan".to_string(),
            //         username: "beginbot".to_string(),
            //         source: None,
            // }));
            "".to_string()
        }
    };
    
    return Ok(Box::new(reply::with_status(challenge, StatusCode::OK)));
}


// #[async_trait]
// impl HandleTwitchEventSub for TwitchEventSubHandler {
//     async fn post_request(self: Box<Self>, eventsub_body: EventSubRoot) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
//         println!("simple_map = {:?}", eventsub_body);
//         
//         let challenge = match eventsub_body.challenge {
//             Some(challenge) => {
//                     // This is required for EventSub's to work!
//                     // If we don't Twitch's challenge, you don't events
//                     challenge
//             }
//             _ =>  {
//
//                 // let obs_client = obs::create_obs_client().await.unwrap();
//                 obs_source::set_enabled(
//                     "Primary",
//                     "Dalle-Gen-1",
//                     true,
//                     &self.obs_client,
//                 ).await;
//                 
//                 // let _ = self.tx.send(Event::UberDuckRequest(subd_types::UberDuckRequest {
//                 //         message: "woah there budy".to_string(),
//                 //         voice_text: "waoh there buddy".to_string(),
//                 //         voice: "Ethan".to_string(),
//                 //         username: "beginbot".to_string(),
//                 //         source: None,
//                 // }));
//                 "".to_string()
//             }
//         };
//         
//         return Ok(Box::new(reply::with_status(challenge, StatusCode::OK)));
//     }
// }
//     
// #[async_trait]
// pub trait HandleTwitchEventSub: Send {
//     async fn post_request(
//         self: Box<Self>,
//         eventsub_body: EventSubRoot,
//         // tx: broadcast::Sender<Event>,
//         // mut rx: broadcast::Receiver<Event>,
//     ) -> Result<Box<dyn warp::Reply>, warp::Rejection>;
// }
