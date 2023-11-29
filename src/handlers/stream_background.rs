use obws::Client as OBSClient;
use anyhow::Result;
use subd_types::Event;
use async_trait::async_trait;
use crate::stream_background_routing;
use tokio::sync::broadcast;
use events::EventHandler;

pub struct StreamBackgroundHandler {
    pub obs_client: OBSClient,
}

#[async_trait]
impl EventHandler for StreamBackgroundHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };
            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            match stream_background_routing::handle_stream_background_commands(
                &tx,
                &self.obs_client,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}
