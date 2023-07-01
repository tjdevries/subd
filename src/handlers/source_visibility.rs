use anyhow::Result;
use events::EventHandler;
use tokio::sync::broadcast;
use subd_types::Event;
use obws::Client as OBSClient;
use async_trait::async_trait;

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
                _ => continue, }; let _ = crate::obs_source::set_enabled(
                &msg.scene,
                &msg.source,
                msg.enabled,
                &self.obs_client,
            )
            .await;
        }
    }
}
