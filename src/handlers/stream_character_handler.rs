use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use subd_types::Event;
use tokio::sync::broadcast;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct StreamCharacterHandler {
    pub obs_client: OBSClient,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventSubRoot {
    subscription: Subscription,
    event: Option<SubEvent>,
    challenge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Subscription {
    id: String,
    status: String,
    #[serde(rename = "type")]
    type_field: String,
    version: String,
    condition: HashMap<String, String>,
    transport: Transport,
    created_at: String,
    cost: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubEvent {
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    tier: Option<String>,
    is_gift: Option<bool>,
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
            let _msg = match event {
                Event::StreamCharacterRequest(msg) => msg,
                _ => continue,
            };

            // let _ = crate::obs_combo::trigger_character_filters(
            //     &msg.source,
            //     &self.obs_client,
            //     msg.enabled,
            // )
            // .await;
        }
    }
}
