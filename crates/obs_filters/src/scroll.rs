use serde::{Deserialize, Serialize};
// use three_d_transform;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollSettings {
    #[serde(rename = "speed_x")]
    pub speed_x: Option<f32>,

    #[serde(rename = "speed_y")]
    pub speed_y: Option<f32>,
}
