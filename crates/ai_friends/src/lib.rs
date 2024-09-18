use anyhow::{Context, Result};
use bytes::Bytes;
use chrono::Utc;
use tokio::fs::create_dir_all;

use fal_ai;

pub async fn sync_lips_to_voice(
    image_file_path: &str,
    audio_file_path: &str,
) -> Result<Bytes> {
    let fal_source_image_data_uri =
        fal_ai::utils::encode_file_as_data_uri(image_file_path).await?;
    let fal_driven_audio_data_uri =
        fal_ai::utils::encode_file_as_data_uri(audio_file_path).await?;

    let fal_result = fal_ai::fal_submit_sadtalker_request(
        &fal_source_image_data_uri,
        &fal_driven_audio_data_uri,
    )
    .await?;
    println!("Sadtalker Result: {}", fal_result);

    let video_url =
        fal_ai::utils::extract_video_url_from_fal_result(&fal_result)?;
    let video_bytes = fal_ai::utils::download_video(&video_url).await?;

    let timestamp = Utc::now().timestamp();
    let video_path = format!(
        "{}/{}.mp4",
        subd_types::consts::get_ai_videos_dir(),
        timestamp
    );
    create_dir_all(subd_types::consts::get_ai_videos_dir()).await?;
    tokio::fs::write(&video_path, &video_bytes)
        .await
        .with_context(|| format!("Failed to write video to {}", video_path))?;
    println!("Video saved to {}", video_path);

    Ok(video_bytes)
}
