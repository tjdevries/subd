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

pub const NOTHING: obws::requests::hotkeys::KeyModifiers =
    obws::requests::hotkeys::KeyModifiers {
        shift: false,
        control: false,
        alt: false,
        command: false,
    };

pub async fn trigger_hotkey(key: &str, obs_client: &OBSClient) -> Result<()> {
    let list = obs_client.hotkeys().list().await?;
    println!("OBS Hotkeys {:?}", list);

    // This workes
    // _ = obs_client
    //     .hotkeys()
    //     .trigger_by_name("libobs.hide_scene_item.1")
    //     .await;

    match obs_client
        .hotkeys()
        .trigger_by_sequence(key, SUPER_KEY)
        .await
    {
        Ok(_) => log::info!("Hotkey triggered successfully"),
        Err(e) => log::error!("Error triggering hotkey: {:?}", e),
    }

    Ok(())
}
