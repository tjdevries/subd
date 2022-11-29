#![allow(unused)]

// TODO:
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound

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

use subd_types::Event;
use subd_types::LunchBytesStatus;

use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tracing::info;
use tracing_subscriber;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::ClientConfig;
use twitch_irc::SecureTCPTransport;
use twitch_irc::TwitchIRCClient;

fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
    let twitch_username = subd_types::consts::get_twitch_bot_username();
    ClientConfig::new_simple(StaticLoginCredentials::new(
        twitch_username,
        Some(subd_types::consts::get_twitch_bot_oauth()),
    ))
}

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

async fn handle_twitch_msg(
    _tx: broadcast::Sender<Event>,
    _rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let _conn = subd_db::get_db_pool().await;

    let config = get_chat_config();
    let (_, _client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);

    Ok(())

    // loop {
    //     let event = rx.recv().await?;
    //     let msg = match event {
    //         Event::TwitchChatMessage(msg) => msg,
    //         _ => continue,
    //     };
    //
    //     match splitmsg[0].as_str() {
    //         "!echo" => {
    //             let echo = commands::Echo::try_parse_from(&splitmsg);
    //             if let Ok(echo) = echo {
    //                 let _ = client.say(twitch_username, echo.contents).await;
    //             }
    //         }
    //         "!lb" => {
    //             if splitmsg.len() == 1 {
    //                 continue;
    //             }
    //
    //             let status = get_lb_status();
    //             let mut status = status.lock().unwrap();
    //             let len = status.topics.len() as u32;
    //             let is_moderator = user_roles.is_moderator();
    //             match splitmsg[1].as_str() {
    //                 "reset" if is_moderator => {
    //                     status.enabled = true;
    //                     status.topics = vec![];
    //                 }
    //                 "show" if is_moderator => {
    //                     status.enabled = true;
    //                 }
    //                 "hide" if is_moderator => {
    //                     status.enabled = false;
    //                 }
    //                 "suggest" => {
    //                     let text = splitmsg[2..].join(" ");
    //                     if text.is_inappropriate() {
    //                         let analysis =
    //                             Censor::from_str(text.as_str()).analyze();
    //                         println!("Text Is: {:?} -> {:?}", text, analysis);
    //                         continue;
    //                     }
    //
    //                     status.topics.push(LunchBytesTopic {
    //                         id: len + 1,
    //                         text: splitmsg[2..].join(" "),
    //                         votes: 1,
    //                     })
    //                 }
    //                 "+" | "^" => {
    //                     if splitmsg.len() < 3 {
    //                         continue;
    //                     }
    //
    //                     let id = match splitmsg[2].as_str().parse::<usize>() {
    //                         Ok(id) => id,
    //                         Err(_) => continue,
    //                     };
    //
    //                     if id == 0 {
    //                         continue;
    //                     }
    //
    //                     match status.topics.get_mut(id - 1) {
    //                         Some(topic) => topic.votes += 1,
    //                         None => continue,
    //                     }
    //                 }
    //
    //                 "-" | "v" => {
    //                     if splitmsg.len() < 3 {
    //                         continue;
    //                     }
    //
    //                     let id = match splitmsg[2].as_str().parse::<usize>() {
    //                         Ok(id) => id,
    //                         Err(_) => continue,
    //                     };
    //
    //                     if id == 0 {
    //                         continue;
    //                     }
    //
    //                     match status.topics.get_mut(id - 1) {
    //                         Some(topic) => {
    //                             topic.votes = max(0, topic.votes - 1)
    //                         }
    //                         None => continue,
    //                     }
    //                 }
    //                 _ => {}
    //             };
    //
    //             tx.send(Event::LunchBytesStatus(status.clone()))?;
    //         }
    //         "!raffle" => {
    //             let result = server::raffle::handle(
    //                 &tx,
    //                 &user_id,
    //                 &msg.sender.name,
    //                 &msg.message_text,
    //             )
    //             .await;
    //             if let Err(err) = result {
    //                 say(
    //                     &client,
    //                     format!("Error while doing raffle: {:?}", err),
    //                 )
    //                 .await?;
    //             }
    //         }
    //         _ => {}
    //     };
    //
    //     // TODO(user_roles)
    //     if msg.message_text.starts_with("!reset themesong")
    //         && msg.badges.iter().any(|badge| badge.name == "moderator")
    //     {
    //         if splitmsg.len() != 3 {
    //             say(
    //                 &client,
    //                 "Invalid reset themesong message format. Try: !reset themesong @name",
    //             )
    //             .await?;
    //             continue;
    //         }
    //
    //         themesong::delete_themesong(&mut conn, splitmsg[2].as_str()).await?
    //     }
    //
    //     // TODO: Add !themesong delete so users can just delete their themesong
    //     if msg.message_text.starts_with("!themesong") {
    //         tx.send(Event::ThemesongDownload(ThemesongDownload::Request {
    //             msg: msg.clone(),
    //         }))?;
    //     }
    //
    //     if msg.message_text.starts_with("!set ") {
    //         println!("  ... doing set command: {:?}", msg.message_text);
    //         let set_result = handle_set_command(&mut conn, &client, msg).await;
    //         if let Err(err) = set_result {
    //             println!("Error while setting: {:?}", err);
    //             say(&client, format!("Error while setting: {:?}", err)).await?;
    //         }
    //     }
    // }
}

async fn _handle_set_command<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    _conn: &mut sqlx::PgConnection,
    _client: &TwitchIRCClient<T, L>,
    msg: twitch_irc::message::PrivmsgMessage,
) -> Result<()> {
    let splitmsg = msg
        .message_text
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    // !set github <login>
    if splitmsg[1] == "github" {
        todo!("Github");
        //     println!("  ... split msg: {:?}", splitmsg);
        //
        //     if splitmsg.len() != 3 {
        //         say(client, format!("@{}: !set github <login>", msg.sender.name))
        //             .await?;
        //         return Ok(());
        //     }
        //
        //     let user_id =
        //         subd_db::get_user_from_twitch_user(conn, &msg.sender.id).await?;
        //
        //     let github_login = splitmsg[2].clone();
        //     subd_db::set_github_info_for_user(
        //         conn,
        //         &user_id,
        //         github_login.as_str(),
        //     )
        //     .await?;
        //     say(
        //         client,
        //         format!(
        //             "Succesfully set: twitch {} -> github {}",
        //             msg.sender.name, github_login
        //         ),
        //     )
        //     .await?;
        //
        //     return Ok(());
    }

    // TODO(user_roles)
    if !msg
        .badges
        .iter()
        .any(|f| f.name == "broadcaster" || f.name == "moderator")
    {
        return Err(anyhow!("Not authorized User"));
    }

    // if splitmsg[1] == "themesong" && splitmsg[2] == "unplayed" {
    //     let twitch_user = splitmsg[3].replace("@", "");
    //     let user_id =
    //         match subd_db::get_user_from_twitch_user_name(conn, &twitch_user)
    //             .await?
    //         {
    //             Some(user_id) => user_id,
    //             None => return Ok(()),
    //         };
    //     themesong::mark_themesong_unplayed(conn, &user_id).await?;
    //     println!(
    //         "  Successfully marked themseong unplayed for: {:?}",
    //         twitch_user
    //     );
    // }

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

async fn handle_obs_stuff(
    _tx: broadcast::Sender<Event>,
    mut _rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let mut _conn = subd_db::get_db_pool().await;

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
    _tx: broadcast::Sender<Event>,
    _rx: broadcast::Receiver<Event>,
) -> Result<()> {
    let _conn = subd_db::get_db_pool().await;

    let config = get_chat_config();
    let (_, _client) = TwitchIRCClient::<
        SecureTCPTransport,
        StaticLoginCredentials,
    >::new(config);

    // loop {
    //     let event = rx.recv().await?;
    //     let msg = match event {
    //         Event::ThemesongDownload(ThemesongDownload::Request { msg }) => msg,
    //         _ => continue,
    //     };
    //
    //     let user_id =
    //         subd_db::get_user_from_twitch_user(&mut conn, &msg.sender.id)
    //             .await?;
    //     let user_roles = subd_db::get_user_roles(&mut conn, &user_id).await?;
    //
    //     let splitmsg = msg
    //         .message_text
    //         .split(" ")
    //         .map(|s| s.to_string())
    //         .collect::<Vec<String>>();
    //
    //     if splitmsg.len() == 1 {
    //         say(&client, "format: !themesong <url> 00:00.00 00:00.00").await?;
    //         tx.send(Event::ThemesongDownload(ThemesongDownload::Format {
    //             sender: msg.sender.name.clone(),
    //         }))?;
    //         continue;
    //     } else if splitmsg.len() != 4 {
    //         say(
    //             &client,
    //             "Incorrect themesong format. Required: !themesong <url> 00:00 00:00",
    //         )
    //         .await?;
    //         tx.send(Event::ThemesongDownload(ThemesongDownload::Finish {
    //             display_name: msg.sender.name.clone(),
    //             success: false,
    //         }))?;
    //         continue;
    //     }
    //
    //     if themesong::can_user_access_themesong(&user_roles) {
    //         // Notify that we are starting a download
    //         tx.send(Event::ThemesongDownload(ThemesongDownload::Start {
    //             display_name: msg.sender.name.clone(),
    //         }))?;
    //
    //         match themesong::download_themesong(
    //             &mut conn,
    //             &user_id,
    //             &msg.sender.name,
    //             splitmsg[1].as_str(),
    //             splitmsg[2].as_str(),
    //             splitmsg[3].as_str(),
    //         )
    //         .await
    //         {
    //             Ok(_) => {
    //                 println!("Successfully downloaded themesong");
    //                 tx.send(Event::ThemesongDownload(
    //                     ThemesongDownload::Finish {
    //                         display_name: msg.sender.name.clone(),
    //                         success: true,
    //                     },
    //                 ))?;
    //
    //                 continue;
    //             }
    //             Err(err) => {
    //                 say(&client, format!("Failed to download: {:?}", err))
    //                     .await?;
    //                 tx.send(Event::ThemesongDownload(
    //                     ThemesongDownload::Finish {
    //                         display_name: msg.sender.name.clone(),
    //                         success: false,
    //                     },
    //                 ))?;
    //
    //                 continue;
    //             }
    //         };
    //     } else {
    //         say(
    //             &client,
    //             "You must be a GH Sponsor or sub/mod/VIP to do this",
    //         )
    //         .await?;
    //     }
    // }
    Ok(())
}

// async fn say<
//     T: twitch_irc::transport::Transport,
//     L: twitch_irc::login::LoginCredentials,
// >(
//     client: &TwitchIRCClient<T, L>,
//     msg: impl Into<String>,
// ) -> Result<()> {
//     let twitch_username = subd_types::consts::get_twitch_broadcaster_username();
//     client.say(twitch_username.to_string(), msg.into()).await?;
//     Ok(())
// }

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

    let mut event_loop = events::EventLoop::new();

    let pool = subd_db::get_db_pool().await;

    // Turns twitch IRC things into our message events
    event_loop.push(twitch_chat::TwitchChat::new(
        pool.clone(),
        "teej_dv".to_string(),
    )?);

    // Does stuff with twitch messages
    event_loop.push(twitch_chat::TwitchMessageHandler::new(
        pool.clone(),
        user_service::Service::new(pool.clone()).await,
    ));

    event_loop.run().await?;

    Ok(())
}
