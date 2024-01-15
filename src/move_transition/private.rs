use anyhow::{Context, Result};
use obws::Client as OBSClient;

pub async fn update_filter_and_enable<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &obws::Client,
) -> Result<()> {
    update_filter(source, filter_name, new_settings, &obs_client)
        .await
        .context(format!(
            "Failed to update Filter: {} on Source: {}",
            filter_name, source
        ))?;
    let filter_enabled = obws::requests::filters::SetEnabled {
        source,
        filter: &filter_name,
        enabled: true,
    };
    Ok(obs_client.filters().set_enabled(filter_enabled).await?)
}

async fn update_filter<T: serde::Serialize>(
    source: &str,
    filter_name: &str,
    new_settings: T,
    obs_client: &OBSClient,
) -> Result<()> {
    let new_filter = obws::requests::filters::SetSettings {
        source,
        filter: filter_name,
        settings: Some(new_settings),
        overlay: Some(true),
    };
    Ok(obs_client.filters().set_settings(new_filter).await?)
}
