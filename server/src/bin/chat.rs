#![allow(dead_code)]

// TODO:
// - We could add twitter usernames
//      - See if ppl are following.
//      - Only allow chatters who retweeted my last tweet, everyone else gets timed out
// - Channel points / Channel Redemptions

// - Theme song:
//      - Add a sound
//          - Download the sound locally
//          - Associated sound w/ user_id
//      - Approve/Reject a sound

use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
use either::Either;
use futures::StreamExt;
use irc::client::prelude::*;
use irc::client::Client as IRCClient;
use obws::requests::SceneItemProperties;
use obws::requests::SourceFilterVisibility;
use obws::Client as OBSClient;
use reqwest::Client as ReqwestClient;
use twitch_api2::helix::HelixClient;
use twitch_api2::twitch_oauth2::{AccessToken, UserToken};

#[derive(Debug)]
struct TwitchMessage {
    channel: String,
    user: TwitchUser,
    contents: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TwitchSubscriber {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug)]
struct TwitchUser {
    login: String,
    subscriber: TwitchSubscriber,
}

fn get_username_from_message(message: &Message) -> Option<String> {
    match &message.prefix {
        Some(prefix) => match prefix {
            Prefix::ServerName(_) => None,
            Prefix::Nickname(_, username, _) => Some(username.clone()),
        },
        None => None,
    }
}

fn get_subtier_from_message(message: &Message) -> TwitchSubscriber {
    let x = message
        .tags
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .find_map(|tag| match tag.0.as_str() {
            "subscriber" => match &tag.1 {
                Some(text) => match text.as_str() {
                    "1" => Some(TwitchSubscriber::Tier1),
                    "2" => Some(TwitchSubscriber::Tier2),
                    "3" => Some(TwitchSubscriber::Tier3),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(TwitchSubscriber::Tier0);

    x
}

#[tokio::main]
async fn main() -> Result<()> {
    // let obs_conn = tungstenite::client::connect("ws://192.168.4.22:4444")?;
    // println!("{:?}", obs_conn);

    // Connect to the OBS instance through obs-websocket.
    let obs_client = OBSClient::connect("192.168.4.22", 4444).await?;

    // Get and print out version information of OBS and obs-websocket.
    let version = obs_client.general().get_version().await?;
    println!("{:#?}", version);

    let scene_item_properties = obs_client
        .scene_items()
        .get_scene_item_properties(Some("PC"), Either::Left("PC - Elgato"))
        .await?;

    dbg!(&scene_item_properties);

    // let mut pc_settings = obs_client
    //     .sources()
    //     .get_source_settings::<serde_json::Value>("PC - Elgato", None)
    //     .await?;
    //
    // println!("Settings: {:?}", pc_settings.source_settings);
    // if pc_settings.source_name == "PC - Elgato" {
    //     // return Ok(());
    //     // *pc_settings
    //     //     .source_settings
    //     //     .get_mut(")
    //     //     .unwrap()
    //     //     .get_mut("active")
    //     //     .unwrap() = Value::Bool(true);
    //     pc_settings
    //         .source_settings
    //         .as_object_mut()
    //         .unwrap()
    //         .insert("active".to_string(), Value::Bool(true));
    // }

    // println!("Settings: {:?}", pc_settings.source_settings);
    // obs_client
    //     .sources()
    //     .set_source_settings::<serde_json::Value, serde_json::Value>(
    //         obws::requests::SourceSettings {
    //             source_name: &pc_settings.source_name,
    //             source_type: Some(&pc_settings.source_type),
    //             source_settings: &pc_settings.source_settings,
    //         },
    //     )
    //     .await?;

    // if pc_settings.source_name == "PC - Elgato" {
    //     return Ok(());
    // }
    // pc_settings.source_settings.

    // Turn on/off a filter for a scene
    // obs_client
    //     .sources()
    //     .set_source_filter_visibility(SourceFilterVisibility {
    //         source_name: "PC - Elgato",
    //         filter_name: "ScrollTest",
    //         filter_enabled: true,
    //     })
    //     .await?;

    // let obs_conn = TcpListener::bind(addr)
    // let obs_conn = TcpStream::
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("teej_dv".to_owned()),
        server: Some("irc.twitch.tv".to_owned()),
        port: Some(6667),
        channels: vec!["#teej_dv".to_owned()],
        use_tls: Some(false),
        password: Some(env::var("TWITCH_OAUTH").expect("$TWITCH_OAUTH must be set")),
        ..Config::default()
    };

    let helix: HelixClient<ReqwestClient> = HelixClient::default();
    let token = UserToken::from_existing(
        &helix,
        AccessToken::new(
            env::var("TWITCH_OAUTH")
                .expect("$TWITCH_OAUTH must be set")
                .replace("oauth:", "")
                .to_string(),
        ),
        None, // Refresh Token
        None, // Client Secret
    )
    .await?;

    let mut client = IRCClient::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let mut conn = subd_db::get_handle().await;

    // Tell twitch we would like the additional metadata
    client.send_cap_req(&[
        Capability::Custom("twitch.tv/commands"),
        Capability::Custom("twitch.tv/tags"),
    ])?;

    // client.send_privmsg(target, message)
    // client.send_
    while let Some(message) = stream.next().await.transpose()? {
        match &message.command {
            Command::PRIVMSG(chan, rawmsg) => {
                let msg = TwitchMessage {
                    channel: chan.clone(),
                    user: TwitchUser {
                        login: get_username_from_message(&message).unwrap(),
                        subscriber: get_subtier_from_message(&message),
                    },
                    contents: rawmsg.clone(),
                    timestamp: Utc::now(),
                };

                // let user = helix.get_user_from_login(msg.user.clone(), &token).await;
                // let subs: Vec<helix::subscriptions::BroadcasterSubscription> = helix
                //     .get_broadcaster_subscriptions(&token)
                //     .try_collect()
                //     .await?;
                // println!("Subs: {:?}", subs);

                let twitch_login = &msg.user.login;
                let user_id = subd_db::get_user_from_twitch_user(&mut conn, twitch_login).await?;
                let user = subd_db::get_user(&mut conn, &user_id).await?;

                // TODO: SubscriptonTiers
                let can_control_dog_cam = if msg.user.login == "teej_dv" {
                    true
                } else if msg.user.subscriber > TwitchSubscriber::Tier0 {
                    true
                } else {
                    match &user.github_user {
                        Some(gh_user) => {
                            let val = subd_gh::is_user_sponsoring(gh_user).await?;
                            if val {
                                println!("User is a github sponsor: {:?}", user.github_user);
                            }
                            val
                        }
                        None => false,
                    }
                };

                if msg.contents.starts_with(":show doggo") && can_control_dog_cam {
                    client
                        .send_privmsg("#teej_dv", format!("@{} -> sets doggo", msg.user.login))?;

                    obs_client.scenes().set_current_scene("PC - Dog").await?;
                    // TODO: Start a timer to set it back?
                }

                if msg.contents.starts_with(":show space") {
                    if can_control_dog_cam {
                        client
                            .send_privmsg("#teej_dv", format!("🚀🚀 @{} 🚀🚀", msg.user.login))?;

                        obs_client
                            .sources()
                            .set_source_filter_visibility(SourceFilterVisibility {
                                source_name: "PC - Elgato",
                                filter_name: "SpaceFilter",
                                filter_enabled: true,
                            })
                            .await?;
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

                        obs_client
                            .sources()
                            .set_source_filter_visibility(SourceFilterVisibility {
                                source_name: "PC - Elgato",
                                filter_name: "SpaceFilter",
                                filter_enabled: false,
                            })
                            .await?;
                    } else {
                        client.send_privmsg("#teej_dv", "📻 Houston, we have a problem")?;
                    }
                }

                if msg.contents.starts_with(":hide background") && can_control_dog_cam {
                    let mut to_set = SceneItemProperties::default();
                    // to_set.scene_name = Some("PC");
                    to_set.item = Either::Left("PC - Elgato");
                    to_set.visible = Some(false);
                    obs_client
                        .scene_items()
                        .set_scene_item_properties(to_set)
                        .await?;
                }

                if msg.contents.starts_with(":show background") && can_control_dog_cam {
                    let mut to_set = SceneItemProperties::default();
                    // to_set.scene_name = Some("PC");
                    to_set.item = Either::Left("PC - Elgato");
                    to_set.visible = Some(true);
                    obs_client
                        .scene_items()
                        .set_scene_item_properties(to_set)
                        .await?;
                }

                let count = subd_db::get_message_count_from_today(&mut conn, &user_id).await?;
                if count == 0 {
                    // TODO: We could do something fun sometimes when new ppl come in
                    // client.send_privmsg(
                    //     "#teej_dv",
                    //     format!("Hey @{}, thanks for stopping by<3", msg.user.login),
                    // )?;
                } else {
                    // obs_client.scenes().set_current_scene("PC").await?;
                }

                if msg.contents.starts_with("!set_github") {
                    subd_db::set_github_user_for_user(
                        &mut conn,
                        &user_id,
                        msg.contents.replace("!set_github", "").trim(),
                    )
                    .await
                    .unwrap_or_else(|e| println!("Nice try, didn't work: {:?}", e))
                }

                subd_db::save_twitch_message(&mut conn, &msg.user.login, &msg.contents).await?;
                println!("Saved: {:?}\n", msg);
                // println!(
                //     "We got a message {} {}, from {:?}",
                //     chan, msg, message.prefix
                // );
            }
            _ => {}
        }
        // match &message.tags {
        //     Some(tags) => tags.iter().for_each(|tag| match tag.0.as_str() {
        //     }),
        //     None => {
        //         print!("No Tags: {}", message);
        //     }
        // }
    }

    Ok(())
}
