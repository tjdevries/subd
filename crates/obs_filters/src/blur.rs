use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BlurSetting {
    #[serde(rename = "Commit")]
    pub commit: Option<String>,

    #[serde(rename = "Filter.Blur.Size")]
    pub size: Option<f32>,

    #[serde(rename = "Filter.Blur.StepScale")]
    pub step_scale: Option<bool>,

    #[serde(rename = "Filter.Blur.StepType")]
    pub step_type: Option<String>,

    #[serde(rename = "Filter.Blur.Version")]
    pub version: Option<u64>,
}
