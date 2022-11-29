use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use subd_types::Event;
use tokio::sync::broadcast;

pub struct UserMessageHandler {}

#[async_trait]
impl EventHandler for UserMessageHandler {
    async fn handle(
        self: Box<Self>,
        _: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let msg = match rx.recv().await? {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            println!("msg: {:?}", msg);
        }
    }
}
