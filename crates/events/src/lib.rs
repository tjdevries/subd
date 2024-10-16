use anyhow::Result;
use async_trait::async_trait;
use subd_types::Event;
use tokio::sync::broadcast;

#[async_trait]
pub trait EventHandler: Send {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()>;
}

pub struct EventLoop {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn push<T>(&mut self, handler: T)
    where
        T: EventHandler + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    pub async fn run(self) -> Result<()> {
        let (base_tx, _) = broadcast::channel::<Event>(256);

        let mut channels = vec![];
        for handler in self.handlers {
            let (tx, rx) = (base_tx.clone(), base_tx.subscribe());
            channels.push(tokio::spawn(async move {
                handler.handle(tx, rx).await.expect("pass")
            }));
        }

        for c in channels {
            // Wait for all the channels to be done
            c.await?;
        }

        Ok(())
    }
}
