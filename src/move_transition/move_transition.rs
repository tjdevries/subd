use crate::move_transition::models;
use anyhow::Result;
use obws::Client as OBSClient;
use std::thread;
use std::time::Duration;

pub async fn move_with_move_source<T: serde::Serialize>(
    scene: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    update_move_source_filters(scene, filter_name, new_settings, &obs_client)
        .await?;
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

pub async fn update_and_trigger_move_value_filter(
    source: &str,
    filter_name: &str,
    filter_setting_name: &str,
    filter_value: f32,
    target_filter_name: &str,
    duration: u32,
    value_type: u32,
    obs_client: &OBSClient,
) -> Result<()> {
    // Fetch the current settings of the filter we are going to update and trigger
    let filter_details =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;

    // Parse the settings into a MoveSingleValueSetting struct
    let mut new_settings = match serde_json::from_value::<
        models::MoveSingleValueSetting,
    >(filter_details.settings)
    {
        Ok(val) => val,
        Err(e) => {
            println!("Error: {:?}", e);
            models::MoveSingleValueSetting {
                ..Default::default()
            }
        }
    };

    println!("Target Filter Name: {}", target_filter_name);
    new_settings.filter = target_filter_name.to_string();

    // Update the settings based on what is passed into the function
    new_settings.source = Some(source.to_string());
    new_settings.setting_name = String::from(filter_setting_name);
    new_settings.setting_float = filter_value;
    new_settings.duration = Some(duration);
    new_settings.value_type = value_type;
    new_settings.move_value_type = Some(value_type);

    println!("------------------------");
    println!("\n\n\tFinal Move Transition Settings: {:?}", new_settings);
    println!("------------------------");

    // Create a SetSettings struct & use it to update the OBS settings
    // TODO: Should this moved into the update_move_source_filters function?
    let new_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: &filter_name,
        settings: new_settings,
        overlay: None,
    };
    obs_client.filters().set_settings(new_settings).await?;

    // Pause so the settings can take effect before triggering the filter
    // TODO: Extract out into variable
    thread::sleep(Duration::from_millis(400));

    // Trigger the filter
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    Ok(())
}

pub async fn update_and_trigger_move_values_filter(
    source: &str,
    filter_name: &str,
    mut new_settings: models::MoveMultipleValuesSetting,
    obs_client: &OBSClient,
) -> Result<()> {
    new_settings.move_value_type = 1;
    new_settings.value_type = 1;
    dbg!(&new_settings);
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: Some(true),
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    thread::sleep(Duration::from_millis(400));
    Ok(())
}

// This takes in settings and updates a filter
async fn update_move_source_filters<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &OBSClient,
) -> Result<()> {
    // What ever this serializes too, ain't right for Move Multiple Settings
    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(true),
    };
    obs_client.filters().set_settings(new_filter).await?;

    Ok(())
}
