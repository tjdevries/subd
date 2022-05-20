use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
use irc::proto::{Command, Message as IRCMessage, Prefix};
use reqwest::Client as ReqwestClient;
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

fn get_id_from_message(message: &IRCMessage) -> Option<String> {
    message
        .tags
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .find_map(|tag| match tag.0.as_str() {
            "user-id" => match &tag.1 {
                Some(text) => Some(text.to_string()),
                _ => None,
            },
            _ => None,
        })
}

fn get_subtier_from_message(message: &IRCMessage) -> TwitchSubscriber {
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

impl From<IRCMessage> for TwitchMessage {
    fn from(msg: IRCMessage) -> Self {
        let (chan, rawmsg) = match &msg.command {
            Command::PRIVMSG(chan, rawmsg) => (chan, rawmsg),
            _ => unreachable!("Cannot pass non-priv msgs to TwitchMessage"),
        };

        TwitchMessage {
            channel: chan.clone(),
            user: TwitchUser::new(
                get_id_from_message(&msg).expect("Twitch chat messages must have ID"),
                get_username_from_message(&msg).expect("Twitch chat messages must have username"),
                get_subtier_from_message(&msg),
            ),
            contents: rawmsg.clone(),
            timestamp: Utc::now(),
            irc_message: msg.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TwitchSubscriber {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub subscriber: TwitchSubscriber,

    moderator: Option<bool>,
}

impl TwitchUser {
    pub fn new(id: String, login: String, subscriber: TwitchSubscriber) -> Self {
        Self {
            id,
            login,
            subscriber,
            moderator: None,
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
