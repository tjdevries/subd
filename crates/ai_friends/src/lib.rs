use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use chrono::Utc;
use colored::Colorize;
use obws::Client as OBSClient;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use subd_types::AiScenesRequest;
use tokio::fs::create_dir_all;
// use tokio::fs::File;
// use tokio::io::BufReader;
use tokio::time::{sleep, Duration};
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub async fn trigger_ai_friend(
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    sink: &Sink,
    ai_scene_req: &AiScenesRequest,
    image_file_path: &str,
    local_audio_path: &str,
    friend_name: &str,
) -> Result<()> {
    println!("Syncing Lips and Voice for Image: {:?}", image_file_path);

    match sync_lips_and_update(
        obs_client,
        twitch_client,
        sink,
        image_file_path,
        local_audio_path,
        friend_name,
    )
    .await
    {
        Ok(_) => {
            if let Some(music_bg) = &ai_scene_req.music_bg {
                let _ = send_message(twitch_client, music_bg.clone()).await;
            }
        }
        Err(e) => {
            eprintln!("Error syncing lips and updating: {:?}", e);
        }
    }
    Ok(())
}

pub async fn sync_lips_to_voice(
    image_file_path: &str,
    audio_file_path: &str,
) -> Result<Bytes> {
    let fal_source_image_data_uri =
        subd_image_utils::encode_file_as_data_uri(image_file_path).await?;
    let fal_driven_audio_data_uri =
        subd_image_utils::encode_file_as_data_uri(audio_file_path).await?;

    let fal_result = fal_ai::fal_submit_sadtalker_request(
        &fal_source_image_data_uri,
        &fal_driven_audio_data_uri,
    )
    .await?;

    let video_url =
        fal_ai::utils::extract_video_url_from_fal_result(&fal_result)?;
    let video_bytes = subd_image_utils::download_video(&video_url).await?;
    println!("{} {}", "Sadtalker Video: ".green(), video_url.cyan());

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

async fn sync_lips_and_update(
    obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    sink: &Sink,
    fal_image_file_path: &str,
    fal_audio_file_path: &str,
    friend_name: &str,
) -> Result<()> {
    let source = friend_name;
    let scene = "AIFriends";
    let video_bytes =
        sync_lips_to_voice(fal_image_file_path, fal_audio_file_path).await?;

    let _ = send_message(twitch_client, "!pause").await;
    // We need to save all videos:
    //   What do name them, how do we store the metadata
    //   We could use a postgresql table
    //   ai_friends_videos
    //     - ID UUID
    //     - friend_name TEXT
    //     - content TEXT
    //     - filename
    //     - store the length of the sound file
    // We only save one version of the ai_friend lip-sync
    // We are saving he video
    let timestamp = Utc::now().timestamp();
    let video_path = format!("./ai_assets/{}-{}.mp4", friend_name, timestamp);
    match tokio::fs::write(&video_path, &video_bytes).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing video: {:?}", e);
            return Err(anyhow!("Error writing video: {:?}", e));
        }
    }

    trigger_friend_intro_and_answer(
        obs_client,
        sink,
        scene,
        source,
        &video_path,
    )
    .await
}

async fn trigger_friend_intro_and_answer(
    obs_client: &OBSClient,
    sink: &Sink,
    scene: &str,
    friend_name: &str,
    video_path: &str,
) -> Result<()> {
    // TODO: update this
    let intro_video_path =
        format!("/home/begin/code/subd/ai_assets/{}_intro.mp4", friend_name);
    println!("Upating OBS Source: {}", intro_video_path);
    let _res = obs_service::obs::update_obs_video_source(
        obs_client,
        &intro_video_path,
        scene,
        friend_name,
    )
    .await?;

    // We need to play applause
    let local_audio_path = "/home/begin/code/subd/MP3s/sitcom_laugh.mp3";
    let file = BufReader::new(File::open(local_audio_path)?);
    sink.append(Decoder::new(BufReader::new(file))?);

    println!("Triggering OBS Source: {}", friend_name);
    let _ = obs_service::obs_source::set_enabled(
        scene,
        &friend_name,
        true,
        obs_client,
    )
    .await;

    // Then we need to update the other file
    // Not sure if I have to wait ofr how long to wait
    println!("we are sleeping");
    sleep(Duration::from_secs(5)).await;

    let _res = obs_service::obs::update_obs_video_source(
        obs_client,
        video_path,
        scene,
        friend_name,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_tag::tag;

    #[tokio::test]
    #[tag(fal)]
    async fn test_sitcom() {
        let obs_client = obs_service::obs::create_obs_client().await.unwrap();

        let source = "melkey";
        let scene = "AIFriends";

        let video = "melkey-1728945940.mp4";
        let video_path = format!("/home/begin/code/subd/ai_assets/{}", video);
        println!("Triggering Update");

        let (_stream, stream_handle) = subd_audio::get_output_stream("pulse")
            .expect("Failed to get audio output stream");
        let sink = rodio::Sink::try_new(&stream_handle)
            .map_err(|e| anyhow::anyhow!("Failed to create sink: {}", e))
            .unwrap();
        let res = trigger_friend_intro_and_answer(
            &obs_client,
            &sink,
            scene,
            source,
            &video_path,
        )
        .await;
        println!("Result: {:?}", res);
    }
}
