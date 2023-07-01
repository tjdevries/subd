use anyhow::Result;
use events::EventHandler;
use tokio::sync::broadcast;
use subd_types::Event;
use obws::Client as OBSClient;
use async_trait::async_trait;


pub struct StreamCharacterHandler {
    pub obs_client: OBSClient,
}


#[async_trait]
impl EventHandler for StreamCharacterHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::StreamCharacterRequest(msg) => msg,
                _ => continue,
            };

            let _ = crate::obs_combo::trigger_character_filters(
                &msg.source,
                &self.obs_client,
                msg.enabled,
            )
            .await;
        }
    }
}

