use crate::client::TwitchChat;
use crate::model;
use crate::model::save_twitch_message;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use subd_types::{Event, UserMessage, UserPlatform};
use tokio::sync::broadcast;
use twitch_irc::message::ServerMessage;

pub struct TwitchMessageHandler {
    pool: sqlx::PgPool,
    twitch: twitch_service::Service,
}

impl TwitchMessageHandler {
    pub fn new(pool: sqlx::PgPool, twitch: twitch_service::Service) -> Self {
        Self { pool, twitch }
    }
}

#[async_trait]
impl EventHandler for TwitchChat {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        _: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // Listen for incoming IRC messages from Twitch
        // we send an TwitchChatMessage event
        // which loop handles somewhere
        while let Some(message) = self.incoming.recv().await {
            match message {
                ServerMessage::Privmsg(private) => {
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

#[async_trait]
impl EventHandler for TwitchMessageHandler {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TwitchChatMessage(msg) => msg,
                _ => continue,
            };

            let user_id = model::upsert_twitch_user(
                &self.pool,
                &msg.sender.id,
                &msg.sender.login,
            )
            .await?;

            save_twitch_message(
                &self.pool,
                &user_id,
                UserPlatform::Twitch,
                &msg.text,
            )
            .await?;

            let user_roles =
                self.twitch.update_user_roles(&user_id, &msg.roles).await?;

            // After update the state of the database, we can go ahead
            // and send the user message to the rest of the system.
            tx.send(Event::UserMessage(UserMessage {
                user_id,
                user_name: msg.sender.name,
                roles: user_roles,
                platform: UserPlatform::Twitch,
                contents: msg.text,
            }))?;
        }
    }
}
