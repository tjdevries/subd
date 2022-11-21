use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use subd_types::Event;
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig,
    SecureTCPTransport, TwitchIRCClient,
};

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
                    tx.send(Event::TwitchChatMessage(private))?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
