use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
use irc::proto::{Command, Message as IRCMessage, Prefix};
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use twitch_api2::{
    helix::{
        moderation::{get_moderators, GetModeratorsRequest},
        HelixClient,
    },
    twitch_oauth2::{AccessToken, UserToken},
    types::UserId,
};

const MY_BROADCASTER_ID: &str = "114257969";

#[derive(Debug, Clone)]
pub struct TwitchMessage {
    pub channel: String,
    pub user: TwitchUser,
    pub contents: String,
    pub timestamp: DateTime<Utc>,
    pub color: Option<String>,

    pub irc_message: IRCMessage,
}

fn get_username_from_message(message: &IRCMessage) -> Option<String> {
    match &message.prefix {
        Some(prefix) => match prefix {
            Prefix::ServerName(_) => None,
            Prefix::Nickname(_, username, _) => Some(username.clone()),
        },
        None => None,
    }
}

fn get_tag_contents(message: &IRCMessage, tag_name: &str) -> Option<String> {
    message
        .tags
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .find_map(|tag| match tag.0.as_str() {
            tag_name => match &tag.1 {
                Some(text) => Some(text.to_string()),
                _ => None,
            },
            _ => None,
        })
}

fn get_id_from_message(message: &IRCMessage) -> Option<String> {
    get_tag_contents(message, "user-id")
}

fn get_subtier_from_message(message: &IRCMessage) -> TwitchSubscriber {
    let subscriber_text = get_tag_contents(message, "subscriber");

    match subscriber_text {
        Some(text) => match text.as_str() {
            "1" => TwitchSubscriber::Tier1,
            "2" => TwitchSubscriber::Tier2,
            "3" => TwitchSubscriber::Tier3,
            _ => TwitchSubscriber::Tier0,
        },
        _ => TwitchSubscriber::Tier0,
    }
}

fn get_color_from_message(message: &IRCMessage) -> Option<String> {
    get_tag_contents(message, "color")
}

impl From<&IRCMessage> for TwitchUser {
    fn from(msg: &IRCMessage) -> Self {
        let id = get_id_from_message(&msg).expect("Twitch chat messages must have ID");
        let login =
            get_username_from_message(&msg).expect("Twitch chat messages must have username");
        let subscriber = get_subtier_from_message(&msg);
        // let badges = get_badges_from_message(&msg);

        Self {
            id,
            login,
            subscriber,
            founder: todo!(),
            moderator: todo!(),
        }
    }
}

impl From<IRCMessage> for TwitchMessage {
    fn from(msg: IRCMessage) -> Self {
        let (chan, rawmsg) = match &msg.command {
            Command::PRIVMSG(chan, rawmsg) => (chan, rawmsg),
            _ => unreachable!("Cannot pass non-priv msgs to TwitchMessage"),
        };

        TwitchMessage {
            channel: chan.clone(),
            user: (&msg).into(),
            contents: rawmsg.clone(),
            timestamp: Utc::now(),
            irc_message: msg.clone(),
            color: get_color_from_message(&msg),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub enum TwitchSubscriber {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub subscriber: TwitchSubscriber,

    founder: Option<bool>,
    moderator: Option<bool>,
}

impl TwitchUser {
    pub fn new(id: String, login: String, subscriber: TwitchSubscriber) -> Self {
        Self {
            id,
            login,
            subscriber,
            moderator: None,
            founder: None,
        }
    }
}

pub async fn is_moderator(user_id: &UserId) -> Result<bool> {
    // TODO: This can make a request if necessary
    let client: HelixClient<ReqwestClient> = HelixClient::default();
    let token = UserToken::from_existing(
        &client,
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

    // let user = helix
    //     .get_user_from_login(self.login.clone(), &token)
    //     .await?;

    // let moderator = helix.get_moderators_in_channel_from_id(broadcaster_id, token)
    let mod_request = GetModeratorsRequest::builder()
        .broadcaster_id(MY_BROADCASTER_ID)
        // TODO: Get this from requesting and saving
        .user_id([user_id.clone()])
        .build();

    let response: Vec<get_moderators::Moderator> = client.req_get(mod_request, &token).await?.data;
    dbg!(&response);

    match response.first() {
        Some(moderator) => Ok(&moderator.user_id == user_id),
        None => Ok(false),
    }
}

impl TwitchUser {
    pub async fn is_moderator(&mut self) -> Result<bool> {
        match self.moderator {
            Some(val) => Ok(val),
            None => {
                // Request the value and update it
                let val = is_moderator(&UserId::from("57632769")).await?;

                // Update moderator value (so we don't request constantly)
                // and then save it off for later
                self.moderator = Some(val);
                Ok(val)
            }
        }
    }
}
