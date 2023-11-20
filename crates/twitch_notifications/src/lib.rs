// use anyhow::Result;
// use futures::{SinkExt, StreamExt};
// use subd_types::Event;
// use tokio::sync::broadcast;
// use tracing::info;
// use twitch_api2::pubsub::{self, Topic};
//
// use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
// use serde_json::{json, Value};
// // use futures_util::stream::StreamExt;
//
// const TWITCH_PUBSUB_URL: &str = "wss://pubsub-edge.twitch.tv";
//
// pub async fn handle_twitch_notifications(
//     tx: broadcast::Sender<Event>,
//     // _: broadcast::Receiver<Event>,
// ) -> Result<()> {
//   let (mut ws_stream, _) = connect_async(TWITCH_PUBSUB_URL).await?;
//
//     // Replace with your actual OAuth token and channel ID
//     // let oauth_token = "oauth:u3okb4xzq2sza3s75rwapcbim09ds2";
//     // OG Doesn't work
//     // let oauth_token = "u3okb4xzq2sza3s75rwapcbim09ds2";
//     //
//     // This was generated from Oauth
//     let oauth_token = "y62udr5cnhhxv8olx50ullhdnnzewx";
//
//     let channel_id = 424038378;
//     
//
//
//     // Constructing the LISTEN message with serde_json
//     let listen_message = json!({
//         "type": "LISTEN",
//         // "nonce": generate_nonce(), // Replace with a function that generates a random nonce
//         "data": {
//             "topics": [format!("channel-subscribe-events-v1.{}", channel_id)],
//             "auth_token": oauth_token
//         }
//     }).to_string();
//
//     ws_stream.send(Message::Text(listen_message)).await?;
//
//     while let Some(message) = ws_stream.next().await {
//         match message? {
//             Message::Text(text) => {
//                 let data: Value = serde_json::from_str(&text)?;
//                 println!("Received message: {:?}", data);
//
//                 // Handle subscription events and other messages here
//             }
//             Message::Binary(_) => println!("Received binary data, not handled in this example"),
//             _ => (),
//         }
//     }
//
//     Ok(())
// }

// // Example nonce generator function
// fn generate_nonce() -> String {
//     use rand::{distributions::Alphanumeric, Rng};
//     rand::thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(30)
//         .collect()
// }
//
// pub async fn handle_twitch_notifications2(
//     tx: broadcast::Sender<Event>,
//     // _: broadcast::Receiver<Event>,
// ) -> Result<()> {
//     // Listen to subscriptions as well
//
//     // Is it OK cloning the string here?
//     let channel_id = subd_types::consts::get_twitch_broadcaster_channel_id()
//         .parse::<u32>()
//         .unwrap();
//     println!("We're in! Channel ID: {}", channel_id);
//     
//     let subscriptions =
//         pubsub::channel_subscriptions::ChannelSubscribeEventsV1 { channel_id }
//             .into_topic();
//
//     let redeems = pubsub::channel_points::ChannelPointsChannelV1 { channel_id }
//         .into_topic();
//
//     // Create the topic command to send to twitch
//     let command = pubsub::listen_command(
//         &[redeems, subscriptions],
//         Some(subd_types::consts::get_twitch_broadcaster_raw().as_str()),
//         "",
//     )
//     .expect("serializing failed");
//     
//     println!("Command {}", command);
//     /// Did the website change?????
//     // wss://eventsub.wss.twitch.tv/ws
//
//     // Send the message with your favorite websocket client
//     info!("trying to connect to stream");
//     let (mut ws_stream, resp) =
//         tokio_tungstenite::connect_async("wss://pubsub-edge.twitch.tv")
//             .await
//             .expect("asdfasdfasdf");
//
//     println!("Got a response from connect async {:?}", resp);
//     ws_stream.send(tungstenite::Message::Text(command)).await?;
//     ws_stream
//         .send(tungstenite::Message::Text(
//             r#"{"type": "PING"}"#.to_string(),
//         ))
//         .await?;
//
//     // Woo we got a msg! Text("{\"type\":\"RESPONSE\",\"error\":\"ERR_BADAUTH\",\"nonce\":\"\"}\r\n")
//
//     println!("We are bout to start lookping!");
//     while let Some(msg) = ws_stream.next().await {
//         match msg {
//             Ok(msg) => {
//                 println!("Woo we got a msg! {:?}", msg);
//                 let msg = match msg {
//                     tungstenite::Message::Text(msg) => msg,
//                     _ => continue,
//                 };
//
//                 let parsed = pubsub::Response::parse(msg.as_str())?;
//                 match parsed {
//                     pubsub::Response::Response(resp) => {
//                         info!(response = ?resp, "(current unhandled)");
//                     }
//                     pubsub::Response::Message { data } => {
//                         match data {
//                             pubsub::TopicData::ChannelPointsChannelV1 {
//                                 topic,
//                                 reply,
//                             } => {
//                                 use pubsub::channel_points::ChannelPointsChannelV1Reply;
//                                 info!(topic = ?topic, "channel point redemption");
//                                 match *reply {
//                                     ChannelPointsChannelV1Reply::RewardRedeemed {
//                                         redemption,
//                                         ..
//                                     } => {
//                                         tx.send(Event::TwitchChannelPointsRedeem(redemption))?;
//                                     }
//                                     // ChannelPointsChannelV1Reply::CustomRewardUpdated { timestamp, updated_reward } => todo!(),
//                                     // ChannelPointsChannelV1Reply::RedemptionStatusUpdate { timestamp, redemption } => todo!(),
//                                     // ChannelPointsChannelV1Reply::UpdateRedemptionStatusesFinished { timestamp, progress } => todo!(),
//                                     // ChannelPointsChannelV1Reply::UpdateRedemptionStatusProgress { timestamp, progress } => todo!(),
//                                     // _ => todo!(),
//                                     _ => {}
//                                 }
//                             }
//                             pubsub::TopicData::ChannelSubscribeEventsV1 {
//                                 topic,
//                                 reply,
//                             } => {
//                                 info!(topic = ?topic, "subscription event");
//                                 tx.send(Event::TwitchSubscription(
//                                     (*reply).into(),
//                                 ))?;
//                                 tx.send(Event::RequestTwitchSubCount)?;
//                             }
//                             // pubsub::TopicData::ChatModeratorActions { topic, reply } => todo!(),
//                             // pubsub::TopicData::ChannelBitsEventsV2 { topic, reply } => todo!(),
//                             // pubsub::TopicData::ChannelBitsBadgeUnlocks { topic, reply } => todo!(),
//                             // pubsub::TopicData::AutoModQueue { topic, reply } => todo!(),
//                             // pubsub::TopicData::UserModerationNotifications { topic, reply } => todo!(),
//                             _ => continue,
//                         }
//                     }
//                     pubsub::Response::Pong => continue,
//                     pubsub::Response::Reconnect => todo!(),
//                 }
//             }
//             Err(err) => {
//                 println!("Error in twitch notifications: {:?}", err);
//             }
//         }
//
//         // TODO: Sometimes sub count is a bit late... oh well
//         tx.send(Event::RequestTwitchSubCount)?;
//     }
//
//     println!("Oh no, exiting");
//
//     Ok(())
// }
//
// // =================================================================================================================


use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest, Responder, http::header};
use hmac::{Hmac};
use hmac::Mac;
use sha2::Sha256;
use std::str;
use serde_json::Value;

type HmacSha256 = Hmac<Sha256>;

async fn eventsub_get(req: HttpRequest) -> impl Responder {
    // Logic for GET request
    println!("Request Info: {:?}", req);
    HttpResponse::Ok().json("")
}

pub async fn eventsub_post(req: HttpRequest, body: web::Bytes) -> impl Responder {
    println!("IS THIS WORKING");
    // let secret = get_secret();
    // let message = get_hmac_message(&req, &body);
    // let mut mac = HmacSha256::new_varkey(secret.as_bytes()).expect("HMAC can take key of any size");
    // mac.update(message.as_bytes());
    // let hmac_hex = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    //if verify_message(&hmac_hex, req.headers().get("Twitch-Eventsub-Message-Signature").unwrap().to_str().unwrap()) {
        println!("signatures match");
        let notification: Value = serde_json::from_slice(&body).unwrap();

        match req.headers().get("Twitch-Eventsub-Message-Type").unwrap().to_str().unwrap() {
            "notification" => {
                // Process the event's data
                println!("Event type: {}", notification["subscription"]["type"]);
                HttpResponse::NoContent().finish()
            }
            "webhook_callback_verification" => {
                // HttpResponse::Ok().content_type("text/plain").body(notification["challenge"].as_str().unwrap())
                HttpResponse::Ok().content_type("text/plain").body(notification["challenge"].as_str().unwrap().to_string())

            }
            "revocation" => {
                println!("notifications revoked!");
                HttpResponse::NoContent().finish()
            }
            _ => HttpResponse::NoContent().finish(),
        }
    // } else {
    //     println!("403 - Signatures did not match");
    //     HttpResponse::Forbidden().finish()
    // }
}

// Is this work???
fn get_secret() -> String {
    "hjv0yajkagdn90bha8x7btj07zu54h".to_string() // Replace with your actual secret
}

fn get_hmac_message(req: &HttpRequest, body: &web::Bytes) -> String {
    let id = req.headers().get("Twitch-Eventsub-Message-Id").unwrap().to_str().unwrap();
    let timestamp = req.headers().get("Twitch-Eventsub-Message-Timestamp").unwrap().to_str().unwrap();
    format!("{}{}{}", id, timestamp, str::from_utf8(body).unwrap())
}

fn verify_message(hmac_hex: &str, verify_signature: &str) -> bool {
    hmac_hex == verify_signature
}

// #[actix_rt::main]
pub async fn kickoff_webhook() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/eventsub", web::get().to(eventsub_get))
            .route("/eventsub", web::post().to(eventsub_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
