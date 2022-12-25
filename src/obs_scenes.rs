use crate::obs;
use anyhow::Result;
use obws;

pub async fn change_scene(obs_client: &obws::Client, name: &str) -> Result<()> {
    obs_client.scenes().set_current_program_scene(&name).await?;
    Ok(())
}

pub async fn find_scene(source: &str) -> Result<String> {
    let scene = match source {
        "begin" => obs::DEFAULT_SCENE,
        _ => obs::MEME_SCENE,
    };

    Ok(scene.to_string())
}
