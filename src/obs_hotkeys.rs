use anyhow::Result;
use obws;
use obws::Client as OBSClient;

pub const SUPER_KEY: obws::requests::hotkeys::KeyModifiers =
    obws::requests::hotkeys::KeyModifiers {
        shift: true,
        control: true,
        alt: true,
        command: true,
    };

pub async fn trigger_hotkey(key: &str, obs_client: &OBSClient) -> Result<()> {
    _ = obs_client
        .hotkeys()
        .trigger_by_sequence(key, SUPER_KEY)
        .await;
    Ok(())
}
