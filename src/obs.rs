use anyhow::Result;

pub async fn set_scene(obs_conn: &obws::Client, scene: &str) -> Result<()> {
    obs_conn.scenes().set_current_program_scene(scene).await?;
    Ok(())
}

pub async fn set_audio_status(
    _obs_conn: &obws::Client,
    _name: &str,
    _status: bool,
) -> Result<()> {
    // obs_conn.sources().(name, !status).await?;
    Ok(())
}
