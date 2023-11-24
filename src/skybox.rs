use std::fs;
use std::io::Write;
use anyhow::Result;
use std::collections::HashMap;
use obws::Client as OBSClient;


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

