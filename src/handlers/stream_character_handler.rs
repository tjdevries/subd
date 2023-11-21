use anyhow::Result;
use events::EventHandler;
use tokio::sync::broadcast;
use subd_types::Event;
use obws::Client as OBSClient;
use async_trait::async_trait;

use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use warp::{http::StatusCode, Filter, Rejection, Reply, reply, reply::json, Buf};


pub struct StreamCharacterHandler {
    pub obs_client: OBSClient,
}

// {"subscription":{"id":"ddfd7140-1590-1dda-ca56-61aac70f9be1","status":"enabled","type":"channel.follow","version":"1","condition":{"broadcaster_user_id":"10483564"},"transport":{"method":"webhook","callback":"null"},"created_at":"2023-11-20T22:57:37.179519123Z","cost":0},"event":{"user_id":"25578051","user_login":"testFromUser","user_name":"testFromUser","broadcaster_user_id":"10483564","broadcaster_user_login":"10483564","broadcaster_user_name":"testBroadcaster","followed_at":"2023-11-20T22:57:37.179519123Z"}}
#[derive(Serialize, Deserialize, Debug)]
struct EventSubRoot {
    subscription: Subscription,
    event: Option<SubEvent>,
    challenge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Subscription {
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
struct SubEvent {
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
impl EventHandler for StreamCharacterHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        
    // let route2 = warp::body::content_length_limit(1024 * 1282)
    // .and(warp::body::json())
    // .map(|simple_map: EventSubRoot| {
    //     // Why are we failing in here?
    //     // It's probably because of differnet shapes of data?
    //
    //     // server::obs_source::set_enabled(
    //     //     "Primary",
    //     //     "Dalle-Gen-1",
    //     //     true,
    //     //     &obs_client,
    //     // ).await;
    //         
    //     println!("simple_map = {:?}", simple_map);
    //     let challenge = match simple_map.challenge {
    //         Some(challenge) => {
    //                 challenge
    //         }
    //         _ =>  {
    //                 // Here we need our own OBS Client
    //                 // and postgres pool
    //                 // So we just have to pass this object around!
    //                 // So now we want to handle this message!
    //                 "".to_string()
    //         }
    //         // return warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    //     };
    //     return warp::reply::with_status(challenge, warp::http::StatusCode::OK)
    //     // warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    // });
    // 
    // 
    // let route3 = warp::body::content_length_limit(1024 * 1282)
    // .and(warp::body::json()).and_then(post_request);
    // 
    // // Run the Warp server in a separate async task
    // tokio::spawn(async move {
    //     warp::serve(route3).run(([0, 0, 0, 0], 8081)).await;
    // });

        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::StreamCharacterRequest(msg) => msg,
                _ => continue,
            };

            let _ = crate::obs_combo::trigger_character_filters(
                &msg.source,
                &self.obs_client,
                msg.enabled,
            )
            .await;
        }
    }
}

// How would I get a tx in here!
async fn post_request(simple_map: EventSubRoot) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // println!("We hit post_request");
    // Ok(Box::new("world"))
    // return Ok(reply::with_status(json(&"".to_string()), StatusCode::OK));
    println!("simple_map = {:?}", simple_map);
    let challenge = match simple_map.challenge {
        Some(challenge) => {
                challenge
        }
        _ =>  {
                // let obs_client = server::obs::create_obs_client().await.unwrap();
                // server::obs_source::set_enabled(
                //     "Primary",
                //     "Dalle-Gen-1",
                //     true,
                //     &obs_client,
                // ).await;
            
            // let _ = tx.send(Event::UberDuckRequest(UberDuckRequest {
            //         message: "woah there budy".to_string(),
            //         voice_text: "waoh there buddy".to_string(),
            //         username: "beginbot".to_string(),
            //         source: None,
            //     }));
                // Here we need our own OBS Client
                // and postgres pool
                // So we just have to pass this object around!
                // So now we want to handle this message!
                "".to_string()
        }
        // return warp::reply::with_status(simple_map.challenge, warp::http::StatusCode::OK)
    };
    
    // return warp::reply::with_status(challenge, warp::http::StatusCode::OK)
    return Ok(Box::new(reply::with_status(challenge, StatusCode::OK)));
}

