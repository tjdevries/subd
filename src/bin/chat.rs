#![allow(unused_variables)]
#![allow(dead_code)]

// TODO:
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound

use std::env;
use std::time::Duration;

use anyhow::Result;
use either::Either;
use futures::SinkExt;
use futures::StreamExt;
use obws::requests::SceneItemProperties;
use obws::requests::SourceFilterVisibility;
use obws::Client as OBSClient;
use reqwest::Client as ReqwestClient;
use subd_types::Event;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use twitch_api2::helix::subscriptions::GetBroadcasterSubscriptionsRequest;
use twitch_api2::helix::HelixClient;
use twitch_api2::pubsub;
use twitch_api2::pubsub::Topic;
use twitch_api2::twitch_oauth2::{AccessToken, UserToken};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

const CONNECT_OBS: bool = false;

async fn handle_twitch_msg(
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        let twitch_login = &msg.sender.login;
        println!(
            "Message: {:?} // Emojis: {:?}",
            msg.message_text, msg.emotes
        );

        subd_db::create_twitch_user_chat(&mut conn, &msg.sender.id, &msg.sender.login).await?;
        subd_db::save_twitch_message(&mut conn, &msg.sender.id, &msg.message_text).await?;

        // let user_id = subd_db::get_user_from_twitch_user(&mut conn, twitch_login).await?;
        // let user = subd_db::get_user(&mut conn, &user_id).await?;
        //
        // let count = subd_db::get_message_count_from_today(&mut conn, &user_id).await?;
        // println!("{} messages today", count);

        /*
        if msg.contents.starts_with(":show doggo") && can_control_dog_cam {
            client.send_privmsg("#teej_dv", format!("@{} -> sets doggo", msg.user.login))?;

            // TODO: Start a timer to set it back?
        }

        if msg.contents.starts_with(":show space") {
            if can_control_dog_cam {
                client.send_privmsg("#teej_dv", format!("🚀🚀 @{} 🚀🚀", msg.user.login))?;
            } else {
                client.send_privmsg("#teej_dv", "📻 Houston, we have a problem")?;
            }
        }

        if msg.contents.starts_with(":hide space") {
            if can_control_dog_cam {
                client.send_privmsg(
                    "#teej_dv",
                    format!("'... Landing rocketship' @{}", msg.user.login),
                )?;
            } else {
                client.send_privmsg("#teej_dv", "📻 Houston, we have a problem")?;
            }
        }
        */
    }

    // if msg.contents.starts_with("!set_github") {
    //     subd_db::set_github_user_for_user(
    //         &mut conn,
    //         &user_id,
    //         msg.contents.replace("!set_github", "").trim(),
    //     )
    //     .await
    //     .unwrap_or_else(|e| println!("Nice try, didn't work: {:?}", e))
    // }

    // println!("Saved: {:?}\n", msg);
    // println!(
    //     "We got a message {} {}, from {:?}",
    //     chan, msg, message.prefix
    // );
}

async fn handle_twitch_chat(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    let conn = subd_db::get_handle().await;
    println!("handle_twitch_chat: got conn");

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    // TODO(generalize)
    client.join("teej_dv".to_owned()).unwrap();

    println!("handle_twitch_chat: waiting for msgs...");
    while let Some(message) = incoming_messages.recv().await {
        match message {
            ServerMessage::Privmsg(private) => {
                tx.send(Event::TwitchChatMessage(private))?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn yew_inner_loop(
    stream: TcpStream,
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    // TODO: Better to split stream so that you can read and write at the same time
    // let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")

    // Get the current sub count
    // tx.send(Event::RequestTwitchSubCount)?;

    println!("Looping new yew inner loop");
    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(_) | Event::TwitchSubscriptionCount(_) => {
                ws_stream
                    .send(tungstenite::Message::Text(serde_json::to_string(&event)?))
                    .await?;
            }
            Event::Shutdown => break,
            _ => continue,
        };
    }

    Ok(())
}

async fn handle_yew(tx: broadcast::Sender<Event>, _: broadcast::Receiver<Event>) -> Result<()> {
    // TODO(generalize)
    let ws = TcpListener::bind("192.168.4.97:9001").await?;

    while let Ok((stream, _)) = ws.accept().await {
        let tx_clone = tx.clone();
        let rx_clone = tx.subscribe();

        tokio::spawn(async move {
            yew_inner_loop(stream, tx_clone, rx_clone)
                .await
                .expect("handling inner loop OK")
        });
    }

    Ok(())
}

async fn handle_twitch_sub_count(
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
    // helix: HelixClient<'static, ReqwestClient>,
) -> Result<()> {
    let helix: HelixClient<ReqwestClient> = HelixClient::default();

    let reqwest_client = helix.clone_client();
    let token = UserToken::from_existing(
        &reqwest_client,
        AccessToken::new(
            env::var("TWITCH_OAUTH")
                .expect("$TWITCH_OAUTH must be set")
                .replace("oauth:", "")
                .to_string(),
        ),
        None, // Refresh Token
        None, // Client Secret
    )
    .await
    .unwrap();

    loop {
        let event = rx.recv().await?;
        match event {
            Event::RequestTwitchSubCount => {
                let req = GetBroadcasterSubscriptionsRequest::builder()
                    .broadcaster_id(token.user_id.clone())
                    .first("1".to_string())
                    .build();

                let response = helix.req_get(req, &token).await.expect("yayayaya");
                let subcount = response.total.unwrap();

                tx.send(Event::TwitchSubscriptionCount(subcount as usize))?;
            }
            _ => continue,
        };
    }
}

async fn handle_twitch_notifications(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    // Listen to subscriptions as well
    let subscriptions = pubsub::channel_subscriptions::ChannelSubscribeEventsV1 {
        channel_id: 114257969,
    }
    .into_topic();

    let redeems = pubsub::channel_points::ChannelPointsChannelV1 {
        channel_id: 114257969,
    }
    .into_topic();

    // Create the topic command to send to twitch
    let command = pubsub::listen_command(
        // &[/* chat_mod_actions,  */ subsriptions],
        &[redeems, subscriptions],
        Some(
            env::var("TWITCH_OAUTH")
                .expect("$TWITCH_OAUTH must be set")
                .replace("oauth:", "")
                .as_str(),
        ),
        "randomsetofletters",
    )
    .expect("serializing failed");

    // Send the message with your favorite websocket client

    println!("trying to connect to stream...");
    // let stream = TcpStream::connect("pubsub-edge.twitch.tv:443")
    //     .await
    //     .unwrap();
    println!("part 1");
    let (mut ws_stream, _resp) = tokio_tungstenite::connect_async("wss://pubsub-edge.twitch.tv")
        .await
        .expect("asdfasdfasdf");

    println!("Got a stream??");
    let written = ws_stream.send(tungstenite::Message::Text(command)).await?;
    dbg!(written);

    let ping = ws_stream
        .send(tungstenite::Message::Text(
            r#"{"type": "PING"}"#.to_string(),
        ))
        .await?;
    dbg!(ping);

    // let (write, read) = ws_stream.split();
    // read.ne

    while let Some(msg) = ws_stream.next().await {
        println!("received new msg");
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("... waiting complete");
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

#[tokio::main]
async fn main() -> Result<()> {
    let mut channels = vec![];
    let (base_tx, _) = broadcast::channel::<Event>(256);

    macro_rules! makechan {
        // If it has (tx, rx) as signature, we can just do this
        ($handle_func:ident) => {{
            let (new_tx, new_rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move {
                $handle_func(new_tx, new_rx)
                    .await
                    .expect("this should work")
            }));
        }};

        // Otherwise, run it like this
        (|$new_tx:ident, $new_rx:ident| $impl:block) => {{
            let ($new_tx, $new_rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move { $impl }));
        }};
    }

    makechan!(handle_twitch_chat);
    makechan!(handle_twitch_msg);
    makechan!(handle_yew);
    makechan!(handle_twitch_sub_count);
    makechan!(handle_twitch_notifications);

    if CONNECT_OBS {
        // Connect to the OBS instance through obs-websocket.
        let obs_client = OBSClient::connect("192.168.4.22", 4444).await?;

        // Get and print out version information of OBS and obs-websocket.
        let version = obs_client.general().get_version().await?;
        println!("OBS Connected: {:#?}", version.version);

        // Can ignore the following, they were just things that I had working before
        // that I didn't want to forget about later.
        obs_client.scenes().set_current_scene("PC - Dog").await?;
        obs_client
            .sources()
            .set_source_filter_visibility(SourceFilterVisibility {
                source_name: "PC - Elgato",
                filter_name: "SpaceFilter",
                filter_enabled: true,
            })
            .await?;

        obs_client
            .sources()
            .set_source_filter_visibility(SourceFilterVisibility {
                source_name: "PC - Elgato",
                filter_name: "SpaceFilter",
                filter_enabled: false,
            })
            .await?;

        let mut to_set = SceneItemProperties::default();
        // to_set.scene_name = Some("PC");
        to_set.item = Either::Left("PC - Elgato");
        to_set.visible = Some(false);
        obs_client
            .scene_items()
            .set_scene_item_properties(to_set)
            .await?;
    }

    for c in channels {
        // Wait for all the channels to be done
        c.await?;
    }

    Ok(())
}
