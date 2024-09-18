use anyhow::Result;
use chrono::Utc;
use obs_service::obs_source;
use obws::Client as OBSClient;

pub async fn take_screenshot(
    screenshot_source: String,
    obs_client: &OBSClient,
) -> Result<(String, String)> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename = format!(
        "/home/begin/code/subd/tmp/screenshots/{}",
        unique_identifier
    );
    obs_source::save_screenshot(&obs_client, &screenshot_source, &filename)
        .await
        .map(|_| (filename, unique_identifier))
}
