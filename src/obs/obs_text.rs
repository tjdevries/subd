use anyhow::Result;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MoveTextFilter {
    #[serde(rename = "setting_name")]
    pub setting_name: String,
    #[serde(rename = "value_type")]
    pub value_type: u32,

    #[serde(rename = "setting_text")]
    pub setting_text: String,
    // "setting_name": "text",
    // "setting_text": "Ok NOW",
    // "value_type": 4
    //
    #[serde(rename = "duration")]
    pub duration: Option<u32>,

    #[serde(rename = "custom_duration")]
    pub custom_duration: bool,

    #[serde(rename = "easing_match")]
    pub easing_match: Option<u32>,

    #[serde(rename = "setting_decimals")]
    pub setting_decimals: Option<u32>,

    // "move_value_type": 4,
    #[serde(rename = "move_value_type")]
    pub move_value_type: Option<u32>,
}

// ===============================================================
// == TEXT
// ===============================================================

// So I need a version to update a text value
// start very unspecific
pub async fn update_and_trigger_text_move_filter(
    source: &str,
    filter_name: &str,
    new_text: &String,
    obs_client: &OBSClient,
) -> Result<()> {
    let mut new_settings: MoveTextFilter = Default::default();
    new_settings.move_value_type = Some(4);
    new_settings.setting_name = "text".to_string();
    new_settings.setting_text = new_text.to_string();
    new_settings.value_type = 4;
    new_settings.duration = Some(300);
    new_settings.custom_duration = true;
    new_settings.setting_decimals = Some(1);

    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };

    // println!("Setting new settings for Filter Name: {}", filter_name);
    obs_client.filters().set_settings(new_settings).await?;

    // This fixes the problem
    // TODO: this should be abstracted into a constant
    let ten_millis = time::Duration::from_millis(300);

    thread::sleep(ten_millis);

    // println!("Filter Name: {}", filter_name);
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    Ok(())
}
