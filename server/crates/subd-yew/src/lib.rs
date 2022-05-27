use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct YewTwitchMessage {
    pub twitch_login: String,
    pub color: Option<String>,
    pub contents: String,
}
