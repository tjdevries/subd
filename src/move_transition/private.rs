use anyhow::Result;
use obws::Client as OBSClient;

// This takes in settings and updates a filter
pub async fn update_move_source_filters<T: serde::Serialize>(
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
