use anyhow::Result;
use futures::{SinkExt, StreamExt};
use subd_types::Event;
use tokio::sync::broadcast;
use tracing::info;
use twitch_api2::pubsub::{self, Topic};

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde_json::{json, Value};
// use futures_util::stream::StreamExt;

const TWITCH_PUBSUB_URL: &str = "wss://pubsub-edge.twitch.tv";

pub async fn handle_twitch_notifications(
    tx: broadcast::Sender<Event>,
    // _: broadcast::Receiver<Event>,
) -> Result<()> {
  let (mut ws_stream, _) = connect_async(TWITCH_PUBSUB_URL).await?;

    // Replace with your actual OAuth token and channel ID
    // let oauth_token = "oauth:u3okb4xzq2sza3s75rwapcbim09ds2";
    // OG Doesn't work
    // let oauth_token = "u3okb4xzq2sza3s75rwapcbim09ds2";
    //
    // This was generated from Oauth
    let oauth_token = "oauth:epyjv975ae6llbze6xflrgr37dhq7e";
    let channel_id = 424038378;
    


    // Constructing the LISTEN message with serde_json
    let listen_message = json!({
        "type": "LISTEN",
        // "nonce": generate_nonce(), // Replace with a function that generates a random nonce
        "data": {
            "topics": [format!("channel-subscribe-events-v1.{}", channel_id)],
            "auth_token": oauth_token
        }
    }).to_string();

    ws_stream.send(Message::Text(listen_message)).await?;

    while let Some(message) = ws_stream.next().await {
        match message? {
            Message::Text(text) => {
                let data: Value = serde_json::from_str(&text)?;
                println!("Received message: {:?}", data);

                // Handle subscription events and other messages here
            }
            Message::Binary(_) => println!("Received binary data, not handled in this example"),
            _ => (),
        }
    }

    Ok(())
}

// // Example nonce generator function
// fn generate_nonce() -> String {
//     use rand::{distributions::Alphanumeric, Rng};
//     rand::thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(30)
//         .collect()
// }

pub async fn handle_twitch_notifications2(
    tx: broadcast::Sender<Event>,
    // _: broadcast::Receiver<Event>,
) -> Result<()> {
    // Listen to subscriptions as well

    // Is it OK cloning the string here?
    let channel_id = subd_types::consts::get_twitch_broadcaster_channel_id()
        .parse::<u32>()
        .unwrap();
    println!("We're in! Channel ID: {}", channel_id);
    
    let subscriptions =
        pubsub::channel_subscriptions::ChannelSubscribeEventsV1 { channel_id }
            .into_topic();

    let redeems = pubsub::channel_points::ChannelPointsChannelV1 { channel_id }
        .into_topic();

    // Create the topic command to send to twitch
    let command = pubsub::listen_command(
        &[redeems, subscriptions],
        Some(subd_types::consts::get_twitch_broadcaster_raw().as_str()),
        "",
    )
    .expect("serializing failed");
    
    println!("Command {}", command);
    /// Did the website change?????
    // wss://eventsub.wss.twitch.tv/ws

    // Send the message with your favorite websocket client
    info!("trying to connect to stream");
    let (mut ws_stream, resp) =
        tokio_tungstenite::connect_async("wss://pubsub-edge.twitch.tv")
            .await
            .expect("asdfasdfasdf");

    println!("Got a response from connect async {:?}", resp);
    ws_stream.send(tungstenite::Message::Text(command)).await?;
    ws_stream
        .send(tungstenite::Message::Text(
            r#"{"type": "PING"}"#.to_string(),
        ))
        .await?;

    // Woo we got a msg! Text("{\"type\":\"RESPONSE\",\"error\":\"ERR_BADAUTH\",\"nonce\":\"\"}\r\n")

    println!("We are bout to start lookping!");
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(msg) => {
                println!("Woo we got a msg! {:?}", msg);
                let msg = match msg {
                    tungstenite::Message::Text(msg) => msg,
                    _ => continue,
                };

                let parsed = pubsub::Response::parse(msg.as_str())?;
                match parsed {
                    pubsub::Response::Response(resp) => {
                        info!(response = ?resp, "(current unhandled)");
                    }
                    pubsub::Response::Message { data } => {
                        match data {
                            pubsub::TopicData::ChannelPointsChannelV1 {
                                topic,
                                reply,
                            } => {
                                use pubsub::channel_points::ChannelPointsChannelV1Reply;
                                info!(topic = ?topic, "channel point redemption");
                                match *reply {
                                    ChannelPointsChannelV1Reply::RewardRedeemed {
                                        redemption,
                                        ..
                                    } => {
                                        tx.send(Event::TwitchChannelPointsRedeem(redemption))?;
                                    }
                                    // ChannelPointsChannelV1Reply::CustomRewardUpdated { timestamp, updated_reward } => todo!(),
                                    // ChannelPointsChannelV1Reply::RedemptionStatusUpdate { timestamp, redemption } => todo!(),
                                    // ChannelPointsChannelV1Reply::UpdateRedemptionStatusesFinished { timestamp, progress } => todo!(),
                                    // ChannelPointsChannelV1Reply::UpdateRedemptionStatusProgress { timestamp, progress } => todo!(),
                                    // _ => todo!(),
                                    _ => {}
                                }
                            }
                            pubsub::TopicData::ChannelSubscribeEventsV1 {
                                topic,
                                reply,
                            } => {
                                info!(topic = ?topic, "subscription event");
                                tx.send(Event::TwitchSubscription(
                                    (*reply).into(),
                                ))?;
                                tx.send(Event::RequestTwitchSubCount)?;
                            }
                            // pubsub::TopicData::ChatModeratorActions { topic, reply } => todo!(),
                            // pubsub::TopicData::ChannelBitsEventsV2 { topic, reply } => todo!(),
                            // pubsub::TopicData::ChannelBitsBadgeUnlocks { topic, reply } => todo!(),
                            // pubsub::TopicData::AutoModQueue { topic, reply } => todo!(),
                            // pubsub::TopicData::UserModerationNotifications { topic, reply } => todo!(),
                            _ => continue,
                        }
                    }
                    pubsub::Response::Pong => continue,
                    pubsub::Response::Reconnect => todo!(),
                }
            }
            Err(err) => {
                println!("Error in twitch notifications: {:?}", err);
            }
        }

        // TODO: Sometimes sub count is a bit late... oh well
        tx.send(Event::RequestTwitchSubCount)?;
    }

    println!("Oh no, exiting");

    Ok(())
}
