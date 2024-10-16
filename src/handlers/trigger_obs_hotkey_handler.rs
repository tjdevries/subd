use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obs_service;
use obws::Client as OBSClient;
use subd_types::Event;
use tokio::sync::broadcast;

pub struct TriggerHotkeyHandler {
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for TriggerHotkeyHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TriggerHotkeyRequest(msg) => msg,
                _ => continue,
            };

            obs_service::obs_hotkeys::trigger_hotkey(
                &msg.hotkey,
                &self.obs_client,
            )
            .await?;
        }
    }
}
