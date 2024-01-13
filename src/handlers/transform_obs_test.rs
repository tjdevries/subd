use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use subd_types::Event;
use tokio::sync::broadcast;

pub struct TransformOBSTextHandler {
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for TransformOBSTextHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::TransformOBSTextRequest(msg) => msg,
                _ => continue,
            };

            // Attempting to Transform! Soundboard-Text TransformSoundboard-Text
            // Attempting to Transform! Soundboard-Text TransformSoundboard-Text Hello

            let filter_name = format!("Transform{}", msg.text_source);
            // println!("Attempting to Transform! {} {} {}", &msg.text_source, &filter_name, &msg.message);

            // We are calling the update and move text filter
            // we should see output
            // let _ =
            //     crate::move_transition::update_and_trigger_text_move_filter(
            //         &msg.text_source,
            //         &filter_name,
            //         &msg.message,
            //         &self.obs_client,
            //     )
            //     .await;
        }
    }
}
