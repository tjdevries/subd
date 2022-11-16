#![allow(dead_code)]

// TODO:
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound

use std::cmp::max;
use std::sync::Mutex;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use futures::SinkExt;
use futures::StreamExt;
// use obws::requests::SceneItemProperties;
// use obws::requests::SourceFilterVisibility;
use obws::Client as OBSClient;
use once_cell::sync::OnceCell;
use reqwest::Client as ReqwestClient;
use rustrict::Censor;
use rustrict::CensorStr;
use server::commands;
use server::themesong;
use server::users;
use subd_types::Event;
use subd_types::LunchBytesStatus;
use subd_types::LunchBytesTopic;
use subd_types::ThemesongDownload;
use subd_types::ThemesongPlay;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use twitch_api2::helix::subscriptions::GetBroadcasterSubscriptionsRequest;
use twitch_api2::helix::HelixClient;
use twitch_api2::pubsub;
use twitch_api2::pubsub::Topic;
use twitch_api2::twitch_oauth2::UserToken;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

// TEMP: We will remove this once we have this in the database.
fn get_lb_status() -> &'static Mutex<LunchBytesStatus> {
    static INSTANCE: OnceCell<Mutex<LunchBytesStatus>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        // Not sure what I changed for this to happen
        Mutex::new(LunchBytesStatus {
            // Mutex::new(&LunchBytesStatus {
            enabled: false,
            topics: vec![],
        })
    })
}

#[tracing::instrument(skip(tx, rx))]
async fn handle_twitch_msg(
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    let config = get_chat_config();
    let (_, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::TwitchChatMessage(msg) => msg,
            _ => continue,
        };

        let badges = msg
            .badges
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        info!(sender = %msg.sender.name, badges = %badges, "{}", msg.message_text);

        subd_db::create_twitch_user_chat(
            &mut conn,
            &msg.sender.id,
            &msg.sender.login,
        )
        .await?;
        subd_db::save_twitch_message(
            &mut conn,
            &msg.sender.id,
            &msg.message_text,
        )
        .await?;

        let user_id =
            subd_db::get_user_from_twitch_user(&mut conn, &msg.sender.id)
                .await?;
        let user_roles =
            users::update_user_roles_once_per_day(&mut conn, &user_id, &msg)
                .await?;

        if themesong::should_play_themesong(&mut conn, &user_id).await? {
            println!("  Sending themesong play event...");
            tx.send(Event::ThemesongPlay(ThemesongPlay::Start {
                user_id,
                display_name: msg.sender.name.clone(),
            }))?;
        }

        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let twitch_username =
            subd_types::consts::get_twitch_broadcaster_username();

        match splitmsg[0].as_str() {
            "!echo" => {
                let echo = commands::Echo::try_parse_from(&splitmsg);
                if let Ok(echo) = echo {
                    let _ = client.say(twitch_username, echo.contents).await;
                }
            }
            "!lb" => {
                if splitmsg.len() == 1 {
                    continue;
                }

                let status = get_lb_status();
                let mut status = status.lock().unwrap();
                let len = status.topics.len() as u32;
                let is_moderator = user_roles.is_moderator();
                match splitmsg[1].as_str() {
                    "reset" if is_moderator => {
                        status.enabled = true;
                        status.topics = vec![];
                    }
                    "show" if is_moderator => {
                        status.enabled = true;
                    }
                    "hide" if is_moderator => {
                        status.enabled = false;
                    }
                    "suggest" => {
                        let text = splitmsg[2..].join(" ");
                        if text.is_inappropriate() {
                            let analysis =
                                Censor::from_str(text.as_str()).analyze();
                            println!("Text Is: {:?} -> {:?}", text, analysis);
                            continue;
                        }

                        status.topics.push(LunchBytesTopic {
                            id: len + 1,
                            text: splitmsg[2..].join(" "),
                            votes: 1,
                        })
                    }
                    "+" | "^" => {
                        if splitmsg.len() < 3 {
                            continue;
                        }

                        let id = match splitmsg[2].as_str().parse::<usize>() {
                            Ok(id) => id,
                            Err(_) => continue,
                        };

                        if id == 0 {
                            continue;
                        }

                        match status.topics.get_mut(id - 1) {
                            Some(topic) => topic.votes += 1,
                            None => continue,
                        }
                    }

                    "-" | "v" => {
                        if splitmsg.len() < 3 {
                            continue;
                        }

                        let id = match splitmsg[2].as_str().parse::<usize>() {
                            Ok(id) => id,
                            Err(_) => continue,
                        };

                        if id == 0 {
                            continue;
                        }

                        match status.topics.get_mut(id - 1) {
                            Some(topic) => {
                                topic.votes = max(0, topic.votes - 1)
                            }
                            None => continue,
                        }
                    }
                    _ => {}
                };

                tx.send(Event::LunchBytesStatus(status.clone()))?;
            }
            "!raffle" => {
                let result = server::raffle::handle(
                    &tx,
                    &user_id,
                    &msg.sender.name,
                    &msg.message_text,
                )
                .await;
                if let Err(err) = result {
                    say(
                        &client,
                        format!("Error while doing raffle: {:?}", err),
                    )
                    .await?;
                }
            }
            _ => {}
        };

        // TODO(user_roles)
        if msg.message_text.starts_with("!reset themesong")
            && msg.badges.iter().any(|badge| badge.name == "moderator")
        {
            if splitmsg.len() != 3 {
                say(
                    &client,
                    "Invalid reset themesong message format. Try: !reset themesong @name",
                )
                .await?;
                continue;
            }

            themesong::delete_themesong(&mut conn, splitmsg[2].as_str()).await?
        }

        // TODO: Add !themesong delete so users can just delete their themesong
        if msg.message_text.starts_with("!themesong") {
            tx.send(Event::ThemesongDownload(ThemesongDownload::Request {
                msg: msg.clone(),
            }))?;
        }

        if msg.message_text.starts_with("!set ") {
            println!("  ... doing set command: {:?}", msg.message_text);
            let set_result = handle_set_command(&mut conn, &client, msg).await;
            if let Err(err) = set_result {
                println!("Error while setting: {:?}", err);
                say(&client, format!("Error while setting: {:?}", err)).await?;
            }
        }
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
}

async fn handle_set_command<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    conn: &mut sqlx::SqliteConnection,
    client: &TwitchIRCClient<T, L>,
    msg: twitch_irc::message::PrivmsgMessage,
) -> Result<()> {
    let splitmsg = msg
        .message_text
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    // !set github <login>
    if splitmsg[1] == "github" {
        println!("  ... split msg: {:?}", splitmsg);

        if splitmsg.len() != 3 {
            say(client, format!("@{}: !set github <login>", msg.sender.name))
                .await?;
            return Ok(());
        }

        let user_id =
            subd_db::get_user_from_twitch_user(conn, &msg.sender.id).await?;

        let github_login = splitmsg[2].clone();
        subd_db::set_github_info_for_user(
            conn,
            &user_id,
            github_login.as_str(),
        )
        .await?;
        say(
            client,
            format!(
                "Succesfully set: twitch {} -> github {}",
                msg.sender.name, github_login
            ),
        )
        .await?;

        return Ok(());
    }

    // TODO(user_roles)
    if !msg
        .badges
        .iter()
        .any(|f| f.name == "broadcaster" || f.name == "moderator")
    {
        return Err(anyhow!("Not authorized User"));
    }

    if splitmsg[1] == "themesong" && splitmsg[2] == "unplayed" {
        let twitch_user = splitmsg[3].replace("@", "");
        let user_id =
            match subd_db::get_user_from_twitch_user_name(conn, &twitch_user)
                .await?
            {
                Some(user_id) => user_id,
                None => return Ok(()),
            };
        themesong::mark_themesong_unplayed(conn, &user_id).await?;
        println!(
            "  Successfully marked themseong unplayed for: {:?}",
            twitch_user
        );
    }

    Ok(())
}

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

#[tracing::instrument(skip(tx))]
async fn handle_twitch_chat(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    // Technically, this one just needs to be able to read chat
    // this client won't send anything to chat.
    let config = get_chat_config();
    let (mut incoming_messages, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);
    let twitch_username = subd_types::consts::get_twitch_broadcaster_username();

    client.join(twitch_username.to_owned()).unwrap();

    info!("waiting for messages");
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
    stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    tx.send(Event::RequestTwitchSubCount)?;
    tx.send(Event::LunchBytesStatus(
        get_lb_status().lock().unwrap().clone(),
    ))?;

    // TODO: Better to split stream so that you can read and write at the same time
    // let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")

    println!("Looping new yew inner loop");
    loop {
        let event = rx.recv().await?;
        match event {
            Event::TwitchChatMessage(_)
            | Event::ThemesongDownload(_)
            | Event::TwitchSubscriptionCount(_)
            | Event::LunchBytesStatus(_)
            | Event::RaffleStatus(_)
            | Event::TwitchSubscription(_) => {
                ws_stream
                    .send(tungstenite::Message::Text(serde_json::to_string(
                        &event,
                    )?))
                    .await?;
            }
            Event::Shutdown => break,
            _ => continue,
        };
    }

    Ok(())
}

async fn handle_yew(
    tx: broadcast::Sender<Event>,
    _: broadcast::Receiver<Event>,
) -> Result<()> {
    // TODO(generalize)
    // This needs to localhost
    let ws = TcpListener::bind("192.168.4.97:9001").await?;

    while let Ok((stream, _)) = ws.accept().await {
        let tx_clone = tx.clone();
        let rx_clone = tx.subscribe();

        tokio::spawn(async move {
            match yew_inner_loop(stream, tx_clone, rx_clone).await {
                Ok(_) => {}
                Err(err) => println!("SOME YEW FAILED WITH: {:?}", err),
            };

            ()
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
        // So here's what ain't working
        subd_types::consts::get_twitch_broadcaster_oauth(),
        subd_types::consts::get_twitch_broadcaster_refresh(),
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

                let response = helix
                    .req_get(req, &token)
                    .await
                    .expect("Error Fetching Twitch Subs");
                let subcount = response.total.unwrap();

                tx.send(Event::TwitchSubscriptionCount(subcount as usize))?;
            }
            _ => continue,
        };
    }
}

#[tracing::instrument(skip(tx))]
async fn handle_twitch_notifications(
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

async fn handle_obs_stuff(
    _tx: broadcast::Sender<Event>,
    mut _rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut _conn = subd_db::get_handle().await;

    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    let obs_client =
        OBSClient::connect(obs_websocket_address, obs_websocket_port, Some(""))
            .await?;

    let version = obs_client.general().version().await?;
    println!("OBS version: {:?}", version);
    // obs_client.scenes().set_current_program_scene()

    // loop {
    //     let event = rx.recv().await?;
    //     match event {
    //         Event::ObsSetScene { scene, .. } => {
    //             server::obs::set_scene(&obs_client, scene.as_str()).await?;
    //         }
    //         Event::TwitchChannelPointsRedeem(redemption) => {
    //             println!("Redemption: {:?}", redemption);
    //
    //             match redemption.reward.title.as_ref() {
    //                 "Mandatory Crab Dance" => {
    //                     // TODO: Mute system audio to stream
    //                     // TODO: Mute linux PC audio to stream
    //                     server::obs::set_scene(&obs_client, "PC - Crab Rave")
    //                         .await?;
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         Event::TwitchChatMessage(msg) => {
    //             let splitmsg = msg
    //                 .message_text
    //                 .split(" ")
    //                 .map(|s| s.to_string())
    //                 .collect::<Vec<String>>();
    //
    //             if splitmsg.len() > 0 {
    //                 continue;
    //             }
    //
    //             if splitmsg.len() != 2 {
    //                 continue;
    //             }
    //
    //             println!("Am i getting here?");
    //             match splitmsg[0].as_str() {
    //                 "!mute" => match splitmsg[1].as_str() {
    //                     "on" => {
    //                         server::obs::set_audio_status(
    //                             &obs_client,
    //                             "Mic/Aux",
    //                             false,
    //                         )
    //                         .await?
    //                     }
    //                     "off" => {
    //                         server::obs::set_audio_status(
    //                             &obs_client,
    //                             "Mic/Aux",
    //                             true,
    //                         )
    //                         .await?
    //                     }
    //                     _ => {}
    //                 },
    //                 _ => {}
    //             }
    //         }
    //         Event::ThemesongPlay(ThemesongPlay::Start { user_id, .. }) => {
    //             macro_rules! set_scene {
    //                 ($scene: expr) => {
    //                     server::obs::set_scene(&obs_client, $scene).await?;
    //                     let tx = tx.clone();
    //                     tokio::spawn(async move {
    //                         tokio::time::sleep(Duration::from_secs(10)).await;
    //                         tx.send(Event::ObsSetScene {
    //                             scene: "PC".to_string(),
    //                         })
    //                         .expect("To be able to set to PC");
    //                     });
    //                 };
    //
    //                 ($scene: expr, $time: expr) => {
    //                     server::obs::set_scene(&obs_client, $scene).await?;
    //                     let tx = tx.clone();
    //                     tokio::spawn(async move {
    //                         tokio::time::sleep(Duration::from_secs($time))
    //                             .await;
    //                         tx.send(Event::ObsSetScene {
    //                             scene: "PC".to_string(),
    //                         })
    //                         .expect("To be able to set to PC");
    //                     });
    //                 };
    //             }
    //             let twitch_user =
    //                 subd_db::get_twitch_user_from_user_id(&mut conn, user_id)
    //                     .await?;
    //             match twitch_user.display_name.to_lowercase().as_ref() {
    //                 "theprimeagen" => {
    //                     set_scene!("Prime Dancing");
    //                 }
    //                 "bashbunni" => {
    //                     set_scene!("Themesong Bash");
    //                 }
    //                 "conni2461" => {
    //                     set_scene!("PC - Daylight", 26);
    //                 }
    //                 _ => {
    //                     // server::obs::set_scene(&obs_client, "Prime Dancing").await?;
    //                 }
    //             }
    //         }
    //         Event::Shutdown => {
    //             break;
    //         }
    //         _ => continue,
    //     };
    // }

    Ok(())
}

async fn handle_themesong_download(
    tx: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    let config = get_chat_config();
    let (_, client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);

    loop {
        let event = rx.recv().await?;
        let msg = match event {
            Event::ThemesongDownload(ThemesongDownload::Request { msg }) => msg,
            _ => continue,
        };

        let user_id =
            subd_db::get_user_from_twitch_user(&mut conn, &msg.sender.id)
                .await?;
        let user_roles = subd_db::get_user_roles(&mut conn, &user_id).await?;

        let splitmsg = msg
            .message_text
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if splitmsg.len() == 1 {
            say(&client, "format: !themesong <url> 00:00.00 00:00.00").await?;
            tx.send(Event::ThemesongDownload(ThemesongDownload::Format {
                sender: msg.sender.name.clone(),
            }))?;
            continue;
        } else if splitmsg.len() != 4 {
            say(
                &client,
                "Incorrect themesong format. Required: !themesong <url> 00:00 00:00",
            )
            .await?;
            tx.send(Event::ThemesongDownload(ThemesongDownload::Finish {
                display_name: msg.sender.name.clone(),
                success: false,
            }))?;
            continue;
        }

        if themesong::can_user_access_themesong(&user_roles) {
            // Notify that we are starting a download
            tx.send(Event::ThemesongDownload(ThemesongDownload::Start {
                display_name: msg.sender.name.clone(),
            }))?;

            match themesong::download_themesong(
                &mut conn,
                &user_id,
                &msg.sender.name,
                splitmsg[1].as_str(),
                splitmsg[2].as_str(),
                splitmsg[3].as_str(),
            )
            .await
            {
                Ok(_) => {
                    println!("Successfully downloaded themesong");
                    tx.send(Event::ThemesongDownload(
                        ThemesongDownload::Finish {
                            display_name: msg.sender.name.clone(),
                            success: true,
                        },
                    ))?;

                    continue;
                }
                Err(err) => {
                    say(&client, format!("Failed to download: {:?}", err))
                        .await?;
                    tx.send(Event::ThemesongDownload(
                        ThemesongDownload::Finish {
                            display_name: msg.sender.name.clone(),
                            success: false,
                        },
                    ))?;

                    continue;
                }
            };
        } else {
            say(
                &client,
                "You must be a GH Sponsor or sub/mod/VIP to do this",
            )
            .await?;
        }
    }
}

async fn handle_themesong_play(
    _: broadcast::Sender<Event>,
    mut rx: broadcast::Receiver<Event>,
    sink: &rodio::Sink,
) -> Result<()> {
    let mut conn = subd_db::get_handle().await;

    loop {
        let event = rx.recv().await?;
        let user_id = match event {
            Event::ThemesongPlay(ThemesongPlay::Start { user_id, .. }) => {
                user_id
            }
            _ => continue,
        };

        println!("=> Playing themesong");
        themesong::play_themesong_for_today(&mut conn, &user_id, &sink).await?;
        let twitch_user =
            subd_db::get_twitch_user_from_user_id(&mut conn, user_id).await?;
        match twitch_user.display_name.as_ref() {
            "theprimeagen" => {
                println!("Wow, theprimeagen is here")
            }
            _ => {}
        }
    }
}

async fn say<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    client: &TwitchIRCClient<T, L>,
    msg: impl Into<String>,
) -> Result<()> {
    let twitch_username = subd_types::consts::get_twitch_broadcaster_username();
    client.say(twitch_username.to_string(), msg.into()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::new("chat=debug,server=debug"))
        .without_time()
        .with_target(false)
        .finish()
        .init();

    info!("Starting chat server");

    // let my_filter = filter::filter_fn(|metadata| {
    //     // Only enable spans or events with the target "interesting_things"
    //     metadata.target() == "interesting_things"
    // });
    //
    // tracing_subscriber::registry()
    //     .with(my_layer.with_filter(my_filter))
    //     .init();

    {
        use rustrict::{add_word, Type};

        // You must take care not to call these when the crate is being
        // used in any other way (to avoid concurrent mutation).
        unsafe {
            add_word(format!("vs{}", "code").as_str(), Type::PROFANE);
            add_word("vsc*de", Type::SAFE);
        }
    }

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

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    makechan!(handle_twitch_chat);
    makechan!(handle_twitch_msg);
    makechan!(handle_yew);
    makechan!(handle_twitch_sub_count);
    makechan!(handle_twitch_notifications);
    makechan!(handle_obs_stuff);

    // Themesong functions
    makechan!(handle_themesong_download);
    makechan!(|tx, rx| {
        handle_themesong_play(tx, rx, &sink)
            .await
            .expect("Handles playing themesongs")
    });

    // TOOD: Can add this back when we're testing twitch subs again.
    // {
    //     let x = base_tx.clone();
    //     tokio::spawn(async move {
    //         println!("==> Sleeping...");
    //         tokio::time::sleep(Duration::from_millis(3000)).await;
    //         println!("... SENDING NYX SUB NOTI");
    //         x.send(Event::TwitchSubscription(get_nyx_sub()))
    //             .expect("to send message x 1");
    //
    //         println!("==> Sleeping x 2...");
    //         tokio::time::sleep(Duration::from_millis(3000)).await;
    //         println!("... x 2 SENDING NYX SUB NOTI");
    //         x.send(Event::TwitchSubscription(get_prime_sub()))
    //             .expect("to send message x 2");
    //     });
    // }

    for c in channels {
        // Wait for all the channels to be done
        c.await?;
    }

    Ok(())
}
