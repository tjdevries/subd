use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollSettings {
    #[serde(rename = "speed_x")]
    pub speed_x: Option<f32>,

    #[serde(rename = "speed_y")]
    pub speed_y: Option<f32>,
}
