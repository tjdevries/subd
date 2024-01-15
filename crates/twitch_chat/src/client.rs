use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig,
    SecureTCPTransport, TwitchIRCClient,
};

#[allow(dead_code)]
pub struct TwitchChat {
    pub broadcaster_username: String,
    pub incoming: UnboundedReceiver<ServerMessage>,
    pub client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pub pool: sqlx::PgPool,
}

impl TwitchChat {
    pub fn new(
        pool: sqlx::PgPool,
        broadcaster_username: String,
    ) -> Result<Self> {
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
            pool,
        })
    }
}

pub async fn send_message<
    T: twitch_irc::transport::Transport,
    L: twitch_irc::login::LoginCredentials,
>(
    client: &TwitchIRCClient<T, L>,
    msg: impl Into<String>,
) -> Result<()> {
    let twitch_username = subd_types::consts::get_twitch_broadcaster_username();
    let str_msg = msg.into();
    // We don't know how to chunk without breaking out current program
    // let chunk_size = 500;
    // for chunk in chunk_string(&str_msg, chunk_size) {
    //     let _ = client
    //         .say(twitch_username.to_string(), chunk)
    //         .await?;
    // }
    //

    let _ = client
        .say(twitch_username.to_string(), str_msg.clone())
        .await?;
    println!("Twitch Send Message: {:?}", str_msg);
    Ok(())
}
