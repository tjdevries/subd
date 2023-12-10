use crate::skybox_requests;
use anyhow::Result;
use askama::Template;
use chrono::Utc;
use obws::Client as OBSClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;

static SKYBOX_STATUS_URL: &str =
    "https://backend.blockadelabs.com/api/v1/imagine/requests";

static SKYBOX_STYLES_URL: &str =
    "https://backend.blockadelabs.com/api/v1/skybox/styles";

static SKYBOX_REMIX_URL: &str =
    "https://backend.blockadelabs.com/api/v1/skybox";

#[derive(Serialize, Deserialize, Debug)]
pub struct SkyboxStyle {
    #[serde(rename = "id")]
    pub skybox_style_id: i32,
    pub name: String,
    #[serde(rename = "max-char")]
    pub max_char: u32,
    #[serde(rename = "negative-text-max-char")]
    pub negative_text_max_char: u32,
    pub image: Option<String>,
    pub sort_order: i32,
    pub premium: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterSkyboxStatusResponse {
    request: SkyboxStatusResponse,
}

// TODO: Consider this name
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SkyboxStatusResponse {
    pub id: i32,
    pub obfuscated_id: String,
    pub user_id: i32,
    pub api_key_id: i32,
    pub title: String,
    pub seed: i32,
    pub negative_text: Option<String>,
    pub prompt: String,
    pub username: String,
    pub status: String,
    pub queue_position: i32,
    pub file_url: String,
    pub thumb_url: String,
    pub depth_map_url: String,
    pub remix_imagine_id: Option<i32>,
    pub remix_obfuscated_id: Option<String>,
    #[serde(rename = "isMyFavorite")]
    pub is_my_favorite: bool,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub error_message: Option<String>,
    pub pusher_channel: String,
    pub pusher_event: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub skybox_style_id: i32,
    pub skybox_id: i32,
    pub skybox_style_name: String,
    pub skybox_name: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "skybox.html")] // using the template in this path, relative
pub struct SkyboxTemplate<'a> {
    // the name of the struct can be anything
    pub url: &'a str, // the field name should match the variable name
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

pub async fn check_styles() -> Result<Vec<SkyboxStyle>> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    let requests_url =
        format!("{}?api_key={}", SKYBOX_STYLES_URL, skybox_api_key);
    let client = Client::new();
    let resp = client.get(&requests_url).send().await.unwrap();

    println!("Skybox Styles: {:?}", resp.status());

    // Let's save this json
    let body = resp.text().await?;

    let mut file = File::create("./tmp/skybox_styles.json")?;
    file.write_all(body.as_bytes())?;

    // Try to parse the String as JSON
    match serde_json::from_str::<Vec<SkyboxStyle>>(&body) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            // If parsing fails, print the error and the raw body for debugging
            eprintln!("Failed to parse JSON: {}", e);
            eprintln!("Raw response body: {}", body);
            Err(e.into()) // Convert the serde_json::Error into a reqwest::Error
        }
    }
}

pub async fn styles_for_chat() -> String {
    let default_val = "".to_string();
    let res = check_styles().await;

    match res {
        Ok(styles) => styles
            .iter()
            .map(|s| format!("{} - {},", s.skybox_style_id, s.name))
            .collect::<String>(),
        Err(_) => default_val.clone(),
    }
}

// TODO: refactor this, so we don't
pub async fn check_skybox_status(id: i32) -> Result<SkyboxStatusResponse> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    let requests_url =
        format!("{}/{}?api_key={}", SKYBOX_STATUS_URL, id, skybox_api_key);
    let client = Client::new();
    let resp = client.get(&requests_url).send().await.unwrap();

    println!("Skybox Status: {:?}", resp.status());
    let body = resp.json::<OuterSkyboxStatusResponse>().await?;
    Ok(body.request)
}

// https://api-documentation.blockadelabs.com/api/skybox.html#get-skybox-by-id
pub async fn check_skybox_status_and_save(id: i32) -> Result<()> {
    let request = match check_skybox_status(id).await {
        Ok(skybox_status) => skybox_status,
        Err(e) => {
            println!("Error Checking Skybox Status: {}", e);
            return Ok(());
        }
    };
    let file_url = request.file_url;

    if file_url != "" {
        println!("File URL: {}", file_url);

        let image_data = reqwest::get(file_url.clone())
            .await?
            .bytes()
            .await?
            .to_vec();

        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let unique_identifier = format!("{}_{}", timestamp, request.id);
        let archive_file =
            format!("./archive/skybox/{}.png", unique_identifier);

        let mut file = File::create(archive_file).unwrap();
        file.write_all(&image_data).unwrap();

        let skybox_template = SkyboxTemplate { url: &file_url };
        let new_skybox = "./build/skybox.html";
        let mut file = File::create(new_skybox).unwrap();
        let render = skybox_template.render().unwrap();
        file.write_all(render.as_bytes()).unwrap();

        // we need to save to the DB
        // We need to trigger OBS
    }
    Ok(())
}

pub async fn request_skybox(
    pool: sqlx::PgPool,
    prompt: String,
    style_id: i32,
) -> io::Result<String> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    // https://backend.blockadelabs.com/api/v1/skybox
    let requests_url =
        format!("{}?api_key={}", SKYBOX_REMIX_URL, skybox_api_key);

    let prompt = prompt.trim_start().to_string();

    let post_body = json!({
        "prompt": prompt,
        "skybox_style_id": style_id,
        // "generator": "stable-skybox",
    });

    let client = Client::new();
    let resp = client
        .post(&requests_url)
        .json(&post_body)
        .send()
        .await
        .unwrap();

    let body = resp.text().await.unwrap();
    let bytes = body.as_bytes();

    let skybox_request: SkyboxStatusResponse =
        serde_json::from_str(&body).unwrap();

    let t = Utc::now();
    let response_filepath = format!("./tmp/skybox_{}.json", t);

    let mut file = File::create(response_filepath.clone())?;
    file.write_all(bytes)?;

    let _ = skybox_requests::save_skybox_request(
        &pool,
        skybox_request.id,
        prompt,
        style_id,
        skybox_request.username,
    )
    .await;

    Ok(response_filepath)
}

#[cfg(test)]
mod tests {
    use crate::skybox;

    #[tokio::test]
    async fn test_time() {
        // let _ = skybox::check_skybox_status(9612607).await;
        // let _ =
        //     skybox::request_skybox("a lush magical castle".to_string()).await;
    }
}
