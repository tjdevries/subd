use anyhow::Result;

pub async fn set_scene(obs_conn: &obws::Client, scene: &str) -> Result<()> {
    obs_conn.scenes().set_current_scene(scene).await?;
    Ok(())
}

pub async fn set_audio_status(obs_conn: &obws::Client, name: &str, status: bool) -> Result<()> {
    obs_conn.sources().set_mute(name, !status).await?;
    Ok(())
}
