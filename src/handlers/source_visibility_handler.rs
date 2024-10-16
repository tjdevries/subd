use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use subd_types::Event;
use tokio::sync::broadcast;

pub struct SourceVisibilityHandler {
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for SourceVisibilityHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::SourceVisibilityRequest(msg) => msg,
                _ => continue,
            };
            let _ = obs_service::obs_source::set_enabled(
                &msg.scene,
                &msg.source,
                msg.enabled,
                &self.obs_client,
            )
            .await;
        }
    }
}
