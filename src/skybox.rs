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

static SKYBOX_REMIX_URL: &str =
    "https://backend.blockadelabs.com/api/v1/skybox";

#[derive(Debug, Serialize, Deserialize)]
pub struct OuterSkyboxStatusResponse {
    request: SkyboxStatusResponse,
}

// TODO: Consider this name
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

#[derive(Template)] // this will generate the code...
#[template(path = "skybox.html")] // using the template in this path, relative
struct SkyboxTemplate<'a> {
    // the name of the struct can be anything
    url: &'a str, // the field name should match the variable name
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
    let resp = client.get(&requests_url).send().await.unwrap();

    println!("Skybox Status: {:?}", resp.status());
    // println!("Skybox Text: {:?}", resp.text().await);
    let body = resp.json::<OuterSkyboxStatusResponse>().await?;

    let file_url = body.request.file_url;

    if file_url != "" {
        println!("File URL: {}", file_url);

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

        let skybox_template = SkyboxTemplate { url: &file_url };
        let new_skybox = "./build/skybox.html";
        let mut file = File::create(new_skybox).unwrap();
        let render = skybox_template.render().unwrap();
        file.write_all(render.as_bytes()).unwrap();

        // Where can I save thijs
        println!("{}", skybox_template.render().unwrap());
    }
    Ok(())
}

// TODO: add the logic for this later
#[allow(dead_code)]
#[allow(unused_variables)]
fn find_style_id(words: Vec<&str>) -> i32 {
    // What is a good default style ID
    return 1;
}

pub async fn request_skybox(prompt: String) -> io::Result<String> {
    let skybox_api_key = env::var("SKYBOX_API_KEY").unwrap();

    // https://backend.blockadelabs.com/api/v1/skybox
    let requests_url =
        format!("{}?api_key={}", SKYBOX_REMIX_URL, skybox_api_key);

    // Do we need to trim start
    // orjshould this done before i'ts passed
    let prompt = prompt.trim_start().to_string();

    // Why???
    let words: Vec<&str> = prompt.split_whitespace().collect();

    // This returns a static style currently
    let skybox_style_id = find_style_id(words);

    println!("Generating Skybox w/ Custom Skybox ID: {}", skybox_style_id);

    // return Ok(String::from("this a hack"));

    let post_body = json!({
        "prompt": prompt,
        // "generator": "stable-skybox",
        // "skybox_style_id": skybox_style_id,
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

    let t = Utc::now();
    let response_filepath = format!("./tmp/skybox_{}.json", t);

    // I need a parsed out body, to save
    let mut file = File::create(response_filepath.clone())?;
    file.write_all(bytes)?;

    // let model = skybox::Skybox

    // if we start saving in the DB here

    // We need to parse the response
    // we need to get the idea, and kick off aprocess that checks Skybox every X seconds
    // if our AI generated bg is done
    Ok(response_filepath)
}

#[cfg(test)]
mod tests {
    use crate::skybox;

    #[tokio::test]
    async fn test_time() {
        // let _ = skybox::check_skybox_status(9612607).await;
        let _ =
            skybox::request_skybox("a lush magical castle".to_string()).await;
    }
}
