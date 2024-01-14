use crate::move_transition::models;
use anyhow::Result;
use obws::Client as OBSClient;
use std::thread;
use std::time::Duration;

pub async fn update_and_trigger_move_values_filter_plus_cache(
    source: &str,
    filter_name: &str,
    mut new_settings: models::MoveMultipleValuesSetting,
    obs_client: &OBSClient,
) -> Result<()> {
    new_settings.move_value_type = 1;
    new_settings.value_type = 1;

    // This all needs to be customizable for names of filters
    // First we get all the values from the current Move filters
    let og_filter_settings =
        match obs_client.filters().get(&source, &filter_name).await {
            Ok(val) => Ok(val),
            Err(err) => Err(err),
        }?;
    let j_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-j")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;
    let k_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-k")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;
    let l_filter_settings = match obs_client
        .filters()
        .get(&source, "Perspective-Cache-l")
        .await
    {
        Ok(val) => Ok(val),
        Err(err) => Err(err),
    }?;

    // We then update all the Move filters
    let settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: filter_name,
        settings: new_settings,
        overlay: None,
    };
    let _ = obs_client.filters().set_settings(settings).await;

    let new_last_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-;",
        settings: l_filter_settings.settings,
        overlay: None,
    };
    let new_l_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-l",
        settings: k_filter_settings.settings,
        overlay: None,
    };
    let new_k_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-k",
        settings: j_filter_settings.settings,
        overlay: None,
    };
    let new_j_settings = obws::requests::filters::SetSettings {
        source: &source,
        filter: "Perspective-Cache-j",
        settings: og_filter_settings.settings,
        overlay: None,
    };

    let _ = obs_client.filters().set_settings(new_last_settings).await;
    let _ = obs_client.filters().set_settings(new_l_settings).await;
    let _ = obs_client.filters().set_settings(new_k_settings).await;
    let _ = obs_client.filters().set_settings(new_j_settings).await;

    // Trigger the main move filter filter
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: &source,
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;

    thread::sleep(Duration::from_millis(400));

    // Trigger the filter
    Ok(())
}
