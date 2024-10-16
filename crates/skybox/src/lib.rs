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
use std::io::Write;
use std::path::Path;
pub mod skybox_requests;

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

#[derive(Template)]
#[template(path = "skybox.html")]
pub struct SkyboxTemplate<'a> {
    pub url: &'a str,
}

pub async fn trigger_scene(
    obs_client: &OBSClient,
    filter_name: &str,
    skybox_id: &str,
) -> Result<()> {
    enable_obs_filter(obs_client, filter_name).await?;
    let skybox_path = get_skybox_path(filter_name, skybox_id);
    write_skybox_command(&skybox_path, filter_name)?;
    Ok(())
}

async fn enable_obs_filter(
    obs_client: &OBSClient,
    filter_name: &str,
) -> Result<()> {
    let filter_enabled = obws::requests::filters::SetEnabled {
        source: "Primary",
        filter: filter_name,
        enabled: true,
    };
    obs_client.filters().set_enabled(filter_enabled).await?;
    Ok(())
}

fn get_skybox_path(filter_name: &str, skybox_id: &str) -> String {
    let skybox_id_map = HashMap::from([
        ("office", "2443168"),
        ("office1", "2443168"),
        ("bar1", "2451051"),
        ("bar", "2449796"),
    ]);

    let id = if skybox_id.is_empty() {
        skybox_id_map.get(filter_name).unwrap_or(&"")
    } else {
        skybox_id
    };

    format!(
        "/home/begin/code/BeginGPT/GoBeginGPT/skybox_archive/{}.txt",
        id
    )
}

fn write_skybox_command(skybox_path: &str, filter_name: &str) -> Result<()> {
    let file_path = "/home/begin/code/BeginGPT/tmp/current/move.txt";
    println!("Checking for Archive: {}", skybox_path);

    let content = if Path::new(skybox_path).exists() {
        let url = fs::read_to_string(skybox_path)?;
        format!("{} {}", filter_name, url)
    } else {
        filter_name.to_string()
    };

    write_to_file(file_path, &content)?;
    Ok(())
}

pub fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub async fn check_styles() -> Result<Vec<SkyboxStyle>> {
    let skybox_api_key = env::var("SKYBOX_API_KEY")?;
    let requests_url =
        format!("{}?api_key={}", SKYBOX_STYLES_URL, skybox_api_key);
    let client = Client::new();
    let resp = client.get(&requests_url).send().await?;

    println!("Skybox Styles: {:?}", resp.status());

    let body = resp.text().await?;
    save_styles_json(&body)?;

    serde_json::from_str(&body).map_err(|e| {
        eprintln!("Failed to parse JSON: {}", e);
        eprintln!("Raw response body: {}", body);
        e.into()
    })
}

fn save_styles_json(body: &str) -> Result<()> {
    let mut file = File::create("./tmp/skybox_styles.json")?;
    file.write_all(body.as_bytes())?;
    Ok(())
}

pub async fn styles_for_chat() -> String {
    check_styles()
        .await
        .map(|styles| {
            styles
                .iter()
                .map(|s| format!("{} - {},", s.skybox_style_id, s.name))
                .collect::<String>()
        })
        .unwrap_or_default()
}

pub async fn check_skybox_status(id: i32) -> Result<SkyboxStatusResponse> {
    let skybox_api_key = env::var("SKYBOX_API_KEY")?;
    let requests_url =
        format!("{}/{}?api_key={}", SKYBOX_STATUS_URL, id, skybox_api_key);
    let client = Client::new();
    let resp = client.get(&requests_url).send().await?;

    println!("Skybox Status: {:?}", resp.status());
    let body = resp.json::<OuterSkyboxStatusResponse>().await?;
    Ok(body.request)
}

pub async fn check_skybox_status_and_save(id: i32) -> Result<()> {
    let request = check_skybox_status(id).await?;
    if !request.file_url.is_empty() {
        save_skybox_image(&request).await?;
        render_skybox_template(&request.file_url)?;
    }
    Ok(())
}

async fn save_skybox_image(request: &SkyboxStatusResponse) -> Result<()> {
    let image_data = reqwest::get(&request.file_url)
        .await?
        .bytes()
        .await?
        .to_vec();
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}", timestamp, request.id);
    let archive_file = format!("./archive/skybox/{}.png", unique_identifier);
    fs::write(archive_file, image_data)?;
    Ok(())
}

fn render_skybox_template(file_url: &str) -> Result<()> {
    let skybox_template = SkyboxTemplate { url: file_url };
    let render = skybox_template.render()?;
    fs::write("./build/skybox.html", render)?;
    Ok(())
}

pub async fn request_skybox(
    pool: sqlx::PgPool,
    prompt: String,
    style_id: i32,
) -> Result<String, String> {
    let skybox_api_key = env::var("SKYBOX_API_KEY")
        .map_err(|e| format!("Failed to get SKYBOX_API_KEY: {}", e))?;
    let requests_url =
        format!("{}?api_key={}", SKYBOX_REMIX_URL, skybox_api_key);
    let prompt = prompt.trim_start().to_string();

    let post_body = json!({
        "prompt": prompt,
        "skybox_style_id": style_id,
    });

    let client = Client::new();
    let resp = client
        .post(&requests_url)
        .json(&post_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body = resp.text().await.map_err(|e| e.to_string())?;
    let skybox_request: SkyboxStatusResponse =
        serde_json::from_str(&body).map_err(|e| e.to_string())?;

    let t = Utc::now();
    let response_filepath = format!("./tmp/skybox_{}.json", t);

    save_skybox_response(
        &response_filepath,
        &body,
        &pool,
        &skybox_request,
        prompt,
        style_id,
    )
    .await?;

    Ok(response_filepath)
}

async fn save_skybox_response(
    filepath: &str,
    body: &str,
    pool: &sqlx::PgPool,
    skybox_request: &SkyboxStatusResponse,
    prompt: String,
    style_id: i32,
) -> Result<(), String> {
    fs::write(filepath, body).map_err(|e| e.to_string())?;
    skybox_requests::save_skybox_request(
        pool,
        skybox_request.id,
        prompt,
        style_id,
        skybox_request.username.clone(),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
