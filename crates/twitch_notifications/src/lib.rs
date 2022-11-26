use anyhow::Result;
use futures::{SinkExt, StreamExt};
use subd_types::Event;
use tokio::sync::broadcast;
use tracing::info;
use twitch_api2::pubsub::{self, Topic};

pub async fn handle_twitch_notifications(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    // Listen to subscriptions as well

    // Is it OK cloning the string here?
    let channel_id = subd_types::consts::get_twitch_broadcaster_channel_id()
        .parse::<u32>()
        .unwrap();
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

    // Send the message with your favorite websocket client
    info!("trying to connect to stream");
    let (mut ws_stream, _resp) =
        tokio_tungstenite::connect_async("wss://pubsub-edge.twitch.tv")
            .await
            .expect("asdfasdfasdf");

    ws_stream.send(tungstenite::Message::Text(command)).await?;
    ws_stream
        .send(tungstenite::Message::Text(
            r#"{"type": "PING"}"#.to_string(),
        ))
        .await?;

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(msg) => {
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

    // let ws = TcpListener::bind(TWITCH_PUBSUB_URL.as_str()).await?;
    // let (mut stream, resp) = tungstenite::connect("wss://pubsub-edge.twitch.tv".to_string())?;
    // println!("  Response: {:?}", resp);
    //
    // while let Ok(msg) = stream.read_message() {
    //     match msg {
    //         tungstenite::Message::Text(msg) => {
    //             let parsed = pubsub::Response::parse(msg.as_str())?;
    //             match parsed {
    //                 pubsub::Response::Response(resp) => {
    //                     println!("[handle_twitch_notifications] got new response: {:?}", resp);
    //                 }
    //                 pubsub::Response::Message { data } => {
    //                     // println!("[handle_twitch_notifications] new msg data: {:?}", data);
    //                     match data {
    //                         pubsub::TopicData::ChannelPointsChannelV1 { topic, reply } => {
    //                             println!("POINTS: {:?}", topic);
    //                             tx.send(Event::RequestTwitchSubCount)?;
    //                         }
    //                         pubsub::TopicData::ChannelSubscribeEventsV1 { topic, reply } => {
    //                             println!("SUBSCRIBE: {:?}", topic);
    //                             tx.send(Event::RequestTwitchSubCount)?;
    //                         }
    //                         // pubsub::TopicData::ChatModeratorActions { topic, reply } => todo!(),
    //                         // pubsub::TopicData::ChannelBitsEventsV2 { topic, reply } => todo!(),
    //                         // pubsub::TopicData::ChannelBitsBadgeUnlocks { topic, reply } => todo!(),
    //                         // pubsub::TopicData::AutoModQueue { topic, reply } => todo!(),
    //                         // pubsub::TopicData::UserModerationNotifications { topic, reply } => todo!(),
    //                         _ => continue,
    //                     }
    //                 }
    //                 pubsub::Response::Pong => continue,
    //                 pubsub::Response::Reconnect => todo!(),
    //             }
    //         }
    //         _ => {
    //             println!("unexpected new msg: {:?}", msg);
    //         }
    //     }
    // }
    //
    println!("Oh no, exiting");

    Ok(())
}
