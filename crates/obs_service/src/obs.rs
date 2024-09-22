use anyhow::Result;
use obws;
use obws::Client as OBSClient;

pub async fn create_obs_client() -> Result<OBSClient, obws::Error> {
    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await
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
