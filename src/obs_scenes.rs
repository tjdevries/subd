use crate::obs;
use anyhow::Result;
use obws;

pub async fn change_scene(obs_client: &obws::Client, name: &str) -> Result<()> {
    let result = obs_client.scenes().set_current_program_scene(&name).await;
    let _ = match result {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Failed to change scene: {}", e)),
    };
    Ok(())
}

pub async fn find_scene(source: &str) -> Result<String> {
    let scene = match source {
        "begin" => obs::DEFAULT_SCENE,
        _ => obs::MEME_SCENE,
    };

    Ok(scene.to_string())
}
