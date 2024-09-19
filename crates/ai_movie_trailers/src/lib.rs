use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use chrono::Utc;
use rodio::Decoder;
use std::fs::File;
use std::io::BufReader;
use subd_types;
use subd_types::AiScenesRequest;
use tokio::fs::create_dir_all;
use twitch_chat::client::send_message;

use fal_rust::client::{ClientCredentials, FalClient};
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub mod models;
pub mod utils;

// This should be in the move_trailer command
pub async fn trigger_movie_trailer(
    ai_scene_req: &AiScenesRequest,
    locked_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    local_audio_path: String,
) -> Result<()> {
    if let Some(music_bg) = &ai_scene_req.music_bg {
        let _ = send_message(&locked_client, music_bg.clone()).await;
    }

    // We are supressing a whole bunch of alsa message
    // let backup =
    //     redirect::redirect_stderr().expect("Failed to redirect stderr");

    let (_stream, stream_handle) =
        subd_audio::get_output_stream("pulse").expect("stream handle");
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let file = BufReader::new(File::open(local_audio_path)?);
    sink.append(Decoder::new(BufReader::new(file))?);
    sink.sleep_until_end();
    // redirect::restore_stderr(backup);
    return Ok(());
}
pub async fn sync_lips_to_voice(
    image_file_path: &str,
    audio_file_path: &str,
) -> Result<Bytes> {
    let fal_source_image_data_uri =
        utils::encode_file_as_data_uri(image_file_path).await?;
    let fal_driven_audio_data_uri =
        utils::encode_file_as_data_uri(audio_file_path).await?;

    let fal_result = fal_submit_sadtalker_request(
        &fal_source_image_data_uri,
        &fal_driven_audio_data_uri,
    )
    .await?;
    println!("FAL Result: {}", fal_result);

    let video_url = utils::extract_video_url_from_fal_result(&fal_result)?;
    let video_bytes = utils::download_video(&video_url).await?;

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

pub async fn create_turbo_image_in_folder(
    prompt: String,
    suno_save_folder: &str,
) -> Result<()> {
    let client = FalClient::new(ClientCredentials::from_env());
    let model = "fal-ai/stable-cascade";
    println!("\tCreating image with model: {}", model);

    let res = client
        .run(
            model,
            serde_json::json!({
                "prompt": prompt,
                "image_size": "landscape_16_9",
            }),
        )
        .await
        .map_err(|e| anyhow!("Failed to run FAL Client: {:?}", e))?;

    let raw_json = res
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from FAL response")?;

    let timestamp = Utc::now().timestamp();
    let json_path = format!(
        "{}/{}.json",
        subd_types::consts::get_fal_responses_dir(),
        timestamp
    );
    create_dir_all(subd_types::consts::get_fal_responses_dir()).await?;
    tokio::fs::write(&json_path, &raw_json)
        .await
        .with_context(|| format!("Failed to write JSON to {}", json_path))?;

    utils::process_fal_images_from_json(
        &raw_json,
        &timestamp.to_string(),
        Some(suno_save_folder),
    )
    .await?;

    Ok(())
}

pub async fn create_video_from_image(image_file_path: &str) -> Result<()> {
    let fal_source_image_data_uri =
        utils::encode_file_as_data_uri(image_file_path).await?;
    let client = FalClient::new(ClientCredentials::from_env());

    let response = client
        .run(
            "fal-ai/stable-video",
            serde_json::json!({ "image_url": fal_source_image_data_uri }),
        )
        .await
        .map_err(|e| anyhow!("Failed to run client: {:?}", e))?;

    let body = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    if let Some(url) = json["video"]["url"].as_str() {
        let video_bytes = utils::download_video(url).await?;
        let timestamp = Utc::now().timestamp();
        let filename = format!(
            "{}/{}.mp4",
            subd_types::consts::get_ai_videos_dir(),
            timestamp
        );
        create_dir_all(subd_types::consts::get_ai_videos_dir()).await?;
        tokio::fs::write(&filename, &video_bytes)
            .await
            .with_context(|| {
                format!("Failed to write video to {}", filename)
            })?;
        println!("Video saved to: {}", filename);
    } else {
        return Err(anyhow!("Failed to extract video URL from JSON"));
    }

    Ok(())
}

pub async fn create_turbo_image(prompt: String) -> Result<()> {
    let client = FalClient::new(ClientCredentials::from_env());
    let model = "fal-ai/fast-sdxl";
    println!("\t\tCreating image with model: {}", model);

    let res = client
        .run(
            model,
            serde_json::json!({
                "prompt": prompt,
                "image_size": "landscape_16_9",
            }),
        )
        .await
        .map_err(|e| anyhow!("Error running Fal Client: {:?}", e))?;

    let raw_json = res
        .bytes()
        .await
        .with_context(|| "Failed to get bytes from response")?;

    let timestamp = Utc::now().timestamp();
    let json_path = format!(
        "{}/{}.json",
        subd_types::consts::get_fal_responses_dir(),
        timestamp
    );

    create_dir_all(subd_types::consts::get_fal_responses_dir()).await?;
    tokio::fs::write(&json_path, &raw_json)
        .await
        .with_context(|| format!("Failed to write JSON to {}", json_path))?;

    utils::process_fal_images_from_json(
        &raw_json,
        &timestamp.to_string(),
        None,
    )
    .await?;

    Ok(())
}

async fn fal_submit_sadtalker_request(
    fal_source_image_data_uri: &str,
    fal_driven_audio_data_uri: &str,
) -> Result<String> {
    let fal_client = FalClient::new(ClientCredentials::from_env());
    let response = fal_client
        .run(
            "fal-ai/sadtalker",
            serde_json::json!({
                "source_image_url": fal_source_image_data_uri,
                "driven_audio_url": fal_driven_audio_data_uri,
            }),
        )
        .await
        .map_err(|e| anyhow!("Error running sadtalker {:?}", e))?;

    if response.status().is_success() {
        response
            .text()
            .await
            .map_err(|e| anyhow!("Error getting text: {:?}", e))
    } else {
        Err(anyhow!(
            "FAL request failed with status: {}",
            response.status()
        ))
    }
}
