use crate::voices_routing;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use subd_types::Event;
use tokio::sync::broadcast;

pub struct VoicesHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
}

#[async_trait]
impl EventHandler for VoicesHandler {
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

            match voices_routing::handle_voices_commands(
                &tx,
                &self.obs_client,
                &self.pool,
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
