use anyhow::{anyhow, Result};
use bytes;
use reqwest::Client;
use rodio::{Decoder, Sink};
use sqlx::types::Uuid;
use std::fs::File;
use std::io::BufReader;
use tokio::fs;
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub mod models;

/// Retrieves audio information based on the song ID.
pub async fn get_audio_information(id: &str) -> Result<models::SunoResponse> {
    let base_url = "http://localhost:3000";
    let url = format!("{}/api/get?ids={}", base_url, id);

    let client = Client::new();
    let response = client.get(&url).send().await?;
    let suno_response: Vec<models::SunoResponse> = response.json().await?;

    suno_response
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No audio information found"))
}

#[derive(Default, Debug, serde::Serialize)]
pub struct AudioGenerationData {
    pub prompt: String,
    pub make_instrumental: bool,
    pub wait_audio: bool,
}

/// Plays audio based on the provided song ID.
pub async fn queue_audio(pool: &sqlx::PgPool, id: &str) -> Result<()> {
    let uuid_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!("Invalid UUID {}: {}", id, e))?;
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;

    Ok(())
}

/// Plays audio based on the provided song ID.
pub async fn play_audio(
    pool: &sqlx::PgPool,
    sink: &Sink,
    id: &str,
) -> Result<()> {
    let file_name = format!("ai_songs/{}.mp3", id);
    let mp3 = File::open(&file_name).map_err(|e| {
        anyhow!(
            "Error opening sound file in play_audio {}: {}",
            file_name,
            e
        )
    })?;
    let file = BufReader::new(mp3);
    let uuid_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!("Invalid UUID {}: {}", id, e))?;

    play_sound_instantly(sink, file).await?;
    ai_playlist::mark_song_as_played(pool, uuid_id).await?;

    Ok(())
}
/// Plays audio based on the provided song ID.
pub async fn add_to_playlist_and_play_audio(
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &sqlx::PgPool,
    sink: &Sink,
    id: &str,
    _username: &str,
) -> Result<()> {
    let file_name = format!("ai_songs/{}.mp3", id);
    let mp3 = File::open(&file_name).map_err(|e| {
        anyhow!("Error opening sound file {}: {}", file_name, e)
    })?;
    let file = BufReader::new(mp3);

    let uuid_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!("Invalid UUID {}: {}", id, e))?;

    // This isn't marked as playing right here
    // Here is an example
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
    play_sound_instantly(sink, file).await?;
    ai_playlist::mark_song_as_played(pool, uuid_id).await?;

    Ok(())
}

/// Plays sound instantly by appending it to the sink.
pub async fn play_sound_instantly(
    sink: &Sink,
    file: BufReader<File>,
) -> Result<()> {
    match Decoder::new(file) {
        Ok(decoder) => {
            // I also need to update the database here
            // sink.clear();
            sink.append(decoder);
            Ok(())
        }
        Err(e) => Err(anyhow!("Error decoding sound file: {}", e)),
    }
}

/// Generates audio based on the provided prompt.
pub async fn generate_audio_by_prompt(
    data: AudioGenerationData,
) -> Result<serde_json::Value> {
    let base_url = "http://localhost:3000/api/generate";
    let client = Client::new();

    let response = client
        .post(base_url)
        .json(&data)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let raw_json = response.text().await?;
    let tmp_file_path =
        format!("tmp/suno_responses/{}.json", chrono::Utc::now().timestamp());
    fs::write(&tmp_file_path, &raw_json).await?;
    println!("Raw JSON saved to: {}", tmp_file_path);

    serde_json::from_str::<serde_json::Value>(&raw_json).map_err(Into::into)
}

/// Downloads the song and initiates playback.
pub async fn download_and_play(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    tx: &broadcast::Sender<subd_types::Event>,
    user_name: String,
    id: &String,
) -> Result<()> {
    let id = id.clone();
    let tx = tx.clone();
    let twitch_client = twitch_client.clone();

    // We need to handle the await here
    tokio::spawn(async move {
        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
        loop {
            println!(
                "{} | Attempting to Download song at: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                cdn_url
            );

            // This response here is where we download
            match reqwest::get(&cdn_url).await {
                Ok(response) if response.status().is_success() => {
                    let content = response.text().await.unwrap();
                    if let Err(e) =
                        just_download(content.as_bytes(), id.clone()).await
                    {
                        eprintln!("Error downloading file: {}", e);
                    }

                    //let t = response.text().await.unwrap();
                    //let suno_response: models::SunoResponse =
                    //    serde_json::from_str(&t).expect("Failed to parse JSON");
                    // we need to create the song here
                    //let created_at =
                    //    sqlx::types::time::OffsetDateTime::now_utc();
                    //let song_id = Uuid::parse_str(&id).unwrap();

                    //// This should be the builder
                    //let new_song = ai_playlist::models::ai_songs::Model {
                    //    song_id,
                    //    title: response.title.to_string(),
                    //    tags: response.metadata.tags.to_string(),
                    //    prompt: response.metadata.prompt,
                    //    username: user_name.clone(),
                    //    audio_url: response.audio_url.to_string(),
                    //    lyric: response.lyric,
                    //    gpt_description_prompt: suno_response
                    //        .metadata
                    //        .gpt_description_prompt
                    //        .to_string(),
                    //    last_updated: Some(created_at),
                    //    created_at: Some(created_at),
                    //    downloaded: false,
                    //};
                    //new_song.save(pool).await?;
                    //
                    let _ = tx.send(subd_types::Event::UserMessage(
                        subd_types::UserMessage {
                            user_name: "beginbot".to_string(),
                            contents: format!("!play {}", id),
                            ..Default::default()
                        },
                    ));

                    let info = format!(
                        "@{}'s song {} added to the Queue.",
                        user_name, id
                    );

                    if let Err(e) = send_message(&twitch_client, info).await {
                        eprintln!("Error sending message: {}", e);
                    }

                    break;
                }
                Ok(_) => {
                    // We don't really want to print here
                    // println!("Song not ready yet, retrying in 5 seconds...");
                }
                Err(e) => {
                    eprintln!("Error fetching song: {}", e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
    Ok(())
}

/// Parses the Suno response, saves song information, and initiates download and playback.
pub async fn parse_suno_response_download_and_play(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    tx: &broadcast::Sender<subd_types::Event>,
    json_response: serde_json::Value,
    index: usize,
    user_name: String,
) -> Result<()> {
    let song_data = json_response
        .get(index)
        .ok_or_else(|| anyhow!("No song data at index {}", index))?;

    let suno_response: models::SunoResponse =
        serde_json::from_value(song_data.clone())
            .expect("Failed to parse JSON");

    let created_at = sqlx::types::time::OffsetDateTime::now_utc();
    let song_id = Uuid::parse_str(&suno_response.id)?;

    // This should be the builder
    let new_song = ai_playlist::models::ai_songs::Model {
        song_id,
        title: suno_response.title.to_string(),
        tags: suno_response.metadata.tags.to_string(),
        prompt: suno_response.metadata.prompt,
        username: user_name.clone(),
        audio_url: suno_response.audio_url.to_string(),
        lyric: suno_response.lyric,
        gpt_description_prompt: suno_response
            .metadata
            .gpt_description_prompt
            .to_string(),
        last_updated: Some(created_at),
        created_at: Some(created_at),
        downloaded: false,
    };
    new_song.save(pool).await?;

    let folder_path = format!("tmp/suno_responses/{}", song_id);
    fs::create_dir_all(&folder_path).await?;

    fs::write(
        format!("tmp/suno_responses/{}.json", song_id),
        &json_response.to_string(),
    )
    .await?;

    let _ =
        download_and_play(twitch_client, tx, user_name, &song_id.to_string())
            .await;

    Ok(ai_playlist::mark_song_as_downloaded(pool, song_id).await?)
}

/// Downloads the audio file and saves it locally.
pub async fn just_download(
    content: &[u8],
    // response: reqwest::Response,
    id: String,
) -> Result<BufReader<File>> {
    let file_name = format!("ai_songs/{}.mp3", id);
    let mut file = fs::File::create(&file_name).await?;

    // let content = response.bytes().await?;

    // now is a good time to mark a song as downloaded
    tokio::io::copy(&mut &content[..], &mut file).await?;
    println!("Downloaded audio to: {}", file_name);

    let mp3 = File::open(&file_name).map_err(|e| {
        anyhow!("Error downloading sound file {}: {}", file_name, e)
    })?;
    let file = BufReader::new(mp3);

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // pub async fn get_audio_information(id: &str) -> Result<models::SunoResponse> {
    #[tokio::test]
    async fn test_get_audio_information() {
        let id = "f12dda07-2588-4326-b15b-63dece759c5f";
        let result = get_audio_information(id).await.unwrap();
        assert_eq!(result.status, "complete");
        assert_eq!(result.title, "Street Pyro");
    }

    #[tokio::test]
    async fn test_parsing_suno_json() {
        let f = fs::read_to_string("./test_data/suno_response.json")
            .expect("Failed to open file");
        let suno_responses: Vec<models::SunoResponse> =
            serde_json::from_str(&f).expect("Failed to parse JSON");

        assert!(!suno_responses.is_empty());
        assert_eq!(suno_responses[0].status, "streaming");
        assert_eq!(
            suno_responses[0].id,
            "f12dda07-2588-4326-b15b-63dece759c5f"
        );
        assert_eq!(suno_responses[0].title, "Street Pyro");
        assert_eq!(suno_responses[0].user_id, "");
        assert_eq!(suno_responses[0].play_count, 0);
        assert_eq!(suno_responses[0].image_url, Some("https://cdn2.suno.ai/image_f12dda07-2588-4326-b15b-63dece759c5f.jpeg".to_string()));
        assert_eq!(suno_responses[0].audio_url,
            "https://audiopipe.suno.ai/?item_id=f12dda07-2588-4326-b15b-63dece759c5f",
        );
    }
}
