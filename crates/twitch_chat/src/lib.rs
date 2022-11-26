use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use reqwest::Client as ReqwestClient;
use subd_types::{Event, UserID, UserPlatform};
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};
use twitch_api2::{
    helix::subscriptions::GetBroadcasterSubscriptionsRequest,
    twitch_oauth2::UserToken, HelixClient,
};
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig,
    SecureTCPTransport, TwitchIRCClient,
};

// fn get_chat_config() -> ClientConfig<StaticLoginCredentials> {
//     let twitch_username = subd_types::consts::get_twitch_bot_username();
//     ClientConfig::new_simple(StaticLoginCredentials::new(
//         twitch_username,
//         Some(subd_types::consts::get_twitch_bot_oauth()),
//     ))
// }

#[allow(dead_code)]
pub struct TwitchChat {
    broadcaster_username: String,
    incoming: UnboundedReceiver<ServerMessage>,
    client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

impl TwitchChat {
    pub fn new(broadcaster_username: String) -> Result<Self> {
        // TODO: Should make bot configurable via this too
        let twitch_username = subd_types::consts::get_twitch_bot_username();
        let config = ClientConfig::new_simple(StaticLoginCredentials::new(
            twitch_username,
            Some(subd_types::consts::get_twitch_bot_oauth()),
        ));

        let (incoming, client) = TwitchIRCClient::<
            SecureTCPTransport,
            StaticLoginCredentials,
        >::new(config);

        client.join(broadcaster_username.clone())?;

        Ok(Self {
            broadcaster_username,
            incoming,
            client,
        })
    }
}

#[async_trait]
impl EventHandler for TwitchChat {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        _: broadcast::Receiver<Event>,
    ) -> Result<()> {
        while let Some(message) = self.incoming.recv().await {
            match message {
                ServerMessage::Privmsg(private) => {
                    // TODO: Turn to internal type
                    tx.send(Event::TwitchChatMessage(
                        subd_types::twitch::TwitchMessage::from_msg(private),
                    ))?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

// TwitchDatabaseConn
//  .create_user(...)
//  .save_message(...)

// First message of the day from trash makes our bot send:
//  You have a wife? Honestly thought this account was ran by a high schooler... Freshman in college at best.

pub struct TwitchMessageHandler {
    conn: sqlx::PgConnection,
}

impl TwitchMessageHandler {
    pub fn new(conn: sqlx::PgConnection) -> Self {
        Self { conn }
    }
}

async fn create_new_user(conn: &mut sqlx::PgConnection) -> Result<UserID> {
    let x = sqlx::query!("INSERT INTO users DEFAULT VALUES RETURNING user_id")
        .fetch_one(conn)
        .await?;

    Ok(UserID(x.user_id))
}

async fn upsert_twitch_user(
    conn: &mut sqlx::PgConnection,
    twitch_user_id: &subd_types::TwitchUserID,
    twitch_user_login: &str,
) -> Result<UserID> {
    // TODO: We should create one transaction for this...

    match sqlx::query!(
        "SELECT user_id FROM twitch_users WHERE twitch_user_id = $1",
        twitch_user_id.0
    )
    .fetch_optional(&mut *conn)
    .await?
    {
        Some(twitch_user) => Ok(UserID(twitch_user.user_id)),
        None => {
            let user_id = create_new_user(&mut *conn).await?;

            sqlx::query!(
             "INSERT INTO twitch_users (user_id, twitch_user_id, login, display_name)
                VALUES($1, $2, $3, $4)",
                user_id.0,
                twitch_user_id.0,
                twitch_user_login,
                twitch_user_login
            )
            .execute(&mut *conn)
            .await
            .unwrap();

            Ok(user_id)
        }
    }
}

pub async fn save_twitch_message(
    conn: &mut sqlx::PgConnection,
    user_id: &UserID,
    platform: UserPlatform,
    message: &str,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO user_messages (user_id, platform, contents)
           VALUES ( $1, $2, $3 )"#,
        user_id.0,
        platform as _,
        message
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

#[async_trait]
impl EventHandler for TwitchMessageHandler {
    async fn handle(
        mut self: Box<Self>,
        _: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TwitchChatMessage(msg) => msg,
                _ => continue,
            };

            let user_id = upsert_twitch_user(
                &mut self.conn,
                &msg.sender.id,
                &msg.sender.login,
            )
            .await?;

            save_twitch_message(
                &mut self.conn,
                &user_id,
                UserPlatform::Twitch,
                &msg.text,
            )
            .await?;
        }

        // Ok(())
    }
}

pub async fn get_twitch_sub_count<'a>(
    client: &HelixClient<'a, ReqwestClient>,
    token: UserToken,
) -> usize {
    let req = GetBroadcasterSubscriptionsRequest::builder()
        .broadcaster_id(token.user_id.clone())
        .first("1".to_string())
        .build();

    let response = client
        .req_get(req, &token)
        .await
        .expect("Error Fetching Twitch Subs");

    response.total.unwrap() as usize
}
