use anyhow::Result;
use async_trait::async_trait;
use elevenlabs_api::Elevenlabs;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::*;
use subd_types::Event;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AiScenesHandler {
    pub sink: Sink,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pub elevenlabs: Elevenlabs,
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for AiScenesHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let ai_scene_req = match event {
                Event::AiScenesRequest(msg) => msg,
                _ => continue,
            };

            let _ = ai_scenes_coordinator::run_ai_scene(
                &self.twitch_client,
                &self.obs_client,
                &self.pool,
                &self.elevenlabs,
                &self.sink,
                &ai_scene_req,
            )
            .await;
        }
    }
}
