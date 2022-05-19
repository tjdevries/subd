use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
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

#[derive(Debug)]
pub struct TwitchMessage {
    pub channel: String,
    pub user: TwitchUser,
    pub contents: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TwitchSubscriber {
    Tier0,
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug)]
pub struct TwitchUser {
    pub login: String,
    pub subscriber: TwitchSubscriber,

    moderator: Option<bool>,
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
