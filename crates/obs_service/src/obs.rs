use anyhow::anyhow;
use anyhow::Result;
use obws;
use obws::Client as OBSClient;

pub async fn update_obs_source(
    obs_client: &OBSClient,
    filename: &str,
    scene: &str,
    source: &str,
) -> Result<()> {
    // let scene = "AIFriends".to_string();
    // let source = "music-video".to_string();
    let path = std::fs::canonicalize(filename)?;
    let full_path = path
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("Failed to convert path to string"))?;

    let _ = crate::obs_source::set_enabled(&scene, &source, false, obs_client)
        .await;
    let _ = crate::obs_source::update_video_source(
        obs_client,
        source.to_string(),
        full_path,
    )
    .await;
    let _ =
        crate::obs_source::set_enabled(&scene, &source, true, obs_client).await;

    crate::obs_scenes::change_scene(obs_client, "Movie Trailer").await
}

pub async fn create_obs_client() -> Result<OBSClient> {
    let obs_websocket_port =
        subd_types::consts::get_obs_websocket_port().parse::<u16>()?;
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await
    .map_err(anyhow::Error::from)
}

// TODO: Find the proper home for this
pub async fn print_filter_info(
    source: &str,
    words: &str,
    obs_client: &OBSClient,
) -> Result<String> {
    println!("Finding Filter Details {:?}", words);

    let filter_details = match obs_client.filters().get(source, words).await {
        Ok(details) => details,
        Err(_) => {
            println!("Error Fetching Filter Details: {:?}", words);
            return Ok("".to_string());
        }
    };

    println!("Filter Details {:?}", filter_details);
    Ok(format!("{:?}", filter_details))
}
