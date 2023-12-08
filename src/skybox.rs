use anyhow::Result;
use chrono::Utc;
use std::fs::File;
use serde::{Deserialize, Serialize};
use std::env;
use reqwest::Client;
use obws::Client as OBSClient;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

static SKYBOX_STATUS_URL: &str = "https://backend.blockadelabs.com/api/v1/imagine/requests";

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterSkyboxStatusResponse {
    request: SkyboxStatusResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkyboxStatusResponse {
    id: i32,
    obfuscated_id: String,
    user_id: i32,
    api_key_id: i32,
    title: String,
    seed: i32,
    negative_text: Option<String>,
    prompt: String,
    username: String,
    status: String,
    queue_position: i32,
    file_url: String,
    thumb_url: String,
    depth_map_url: String,
    remix_imagine_id: Option<i32>,
    remix_obfuscated_id: Option<String>,
    #[serde(rename = "isMyFavorite")]
    is_my_favorite: bool,
    #[serde(rename = "created_at")]
    created_at: String,
    #[serde(rename = "updated_at")]
    updated_at: String,
    error_message: Option<String>,
    pusher_channel: String,
    pusher_event: String,
    #[serde(rename = "type")]
    item_type: String,
    skybox_style_id: i32,
    skybox_id: i32,
    skybox_style_name: String,
    skybox_name: String,
}


// OBS_filter_name
// skybox_id
pub async fn trigger_scene(
    obs_client: &OBSClient,
    filter_name: &str,
    skybox_id: &str,
) -> Result<()> {
    let scene = "Primary";
    // TODO: make this dynamic
    // let content = "lunch";

    let filter_enabled = obws::requests::filters::SetEnabled {
        source: scene,
        filter: &filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    let file_path = "/home/begin/code/BeginGPT/tmp/current/move.txt";

    let skybox_id_map = HashMap::from([
        ("office".to_string(), "2443168".to_string()),
        ("office1".to_string(), "2443168".to_lowercase()),
        ("bar1".to_string(), "2451051".to_string()),
        ("bar".to_string(), "2449796".to_string()),
    ]);

    let skybox_path = if skybox_id == "" {
        let new_skybox_id = &skybox_id_map[filter_name.clone()];
        format!(
            "/home/begin/code/BeginGPT/GoBeginGPT/skybox_archive/{}.txt",
            new_skybox_id
        )
    } else {
        format!(
            "/home/begin/code/BeginGPT/GoBeginGPT/skybox_archive/{}.txt",
            skybox_id
        )
    };

    // This URL is rare
    // unless you look up ID based on
    println!("Checking for Archive: {}", skybox_path);
    let skybox_url_exists = std::path::Path::new(&skybox_path).exists();

    if skybox_url_exists {
        let url = fs::read_to_string(skybox_path).expect("Can read file");
        let new_skybox_command = format!("{} {}", &filter_name, url);
        if let Err(e) = write_to_file(file_path, &new_skybox_command) {
            eprintln!("Error writing to file: {}", e);
        }
    } else {
        if let Err(e) = write_to_file(file_path, &filter_name) {
            eprintln!("Error writing to file: {}", e);
        }
    }

    return Ok(());
}

pub fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub async fn check_skybox_status(id: i32) -> Result<()> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();
    
    // https://backend.blockadelabs.com/api/v1/skybox
    // https://api-documentation.blockadelabs.com/api/skybox.html#get-skybox-by-id
    let requests_url =
        format!("{}/{}?api_key={}", SKYBOX_STATUS_URL, id, skybox_api_key);
    let client = Client::new();
    let resp = client
        .get(&requests_url)
        .send()
        .await
        .unwrap();

    println!("Skybox Status: {:?}", resp.status());
    // println!("Skybox Text: {:?}", resp.text().await);
    let body = resp.json::<OuterSkyboxStatusResponse>().await?;

    let file_url = body.request.file_url;
    
    if file_url != "" {
        let image_data = reqwest::get(file_url.clone())
            .await?
            .bytes()
            .await?
            .to_vec();
        
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let unique_identifier = format!("{}_{}", timestamp, body.request.id);
        let archive_file =
            format!("./archive/skybox/{}.png", unique_identifier);
        
        let mut file = File::create(archive_file).unwrap();
        file.write_all(&image_data).unwrap();
    }


                
                //
                //
                // writeln!(csv_file, "{},{}", unique_identifier, contents)
                //     .unwrap();
                //
                // let filename = format!("./tmp/dalle-{}.png", index + 1);
                // let mut file = File::create(filename).unwrap();
                // file.write_all(&image_data).unwrap();
    Ok(())
}
    

#[cfg(test)]
mod tests {
    use crate::skybox;
    
    #[tokio::test]
    async fn test_time() {
       let _ = skybox::check_skybox_status(9612607).await;
       assert_eq!(1, 2);
    }
}
