//use anyhow::anyhow;
//use anyhow::Result;
//use reqwest::Client;
//use rodio::Decoder;
//use rodio::Sink;
//use sqlx::types::Uuid;
//use std::fs::File;
//use std::io::BufReader;
//use std::thread;
//use subd_types::Event;
//use tokio::runtime::Runtime;
//use tokio::sync::broadcast;
//use twitch_chat::client::send_message;
//use twitch_irc::{
//    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
//};
//
//pub mod models;
//
//#[derive(Default, Debug)]
//pub struct AudioGenerationData {
//    pub prompt: String,
//    pub make_instrumental: bool,
//    pub wait_audio: bool,
//}
//
//pub async fn play_audio(
//    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
//    pool: &sqlx::PgPool,
//    sink: &Sink,
//    id: &str,
//    user_name: &str,
//) -> Result<()> {
//    println!("\tQueuing {}", id);
//    let info = format!("@{} added {} to Queue", user_name, id);
//    let _ = send_message(&twitch_client, info).await;
//
//    let file_name = format!("ai_songs/{}.mp3", id);
//    let mp3 = match File::open(&file_name) {
//        Ok(file) => file,
//        Err(e) => {
//            eprintln!("Error opening sound file: {}", e);
//            return Ok(());
//        }
//    };
//    let file = BufReader::new(mp3);
//    println!("\tPlaying Audio {}", id);
//
//    let uuid_id = uuid::Uuid::parse_str(id)?;
//
//    println!("Adding to Playlist");
//    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
//    ai_playlist::mark_song_as_played(pool, uuid_id).await?;
//
//    let _ = play_sound_instantly(sink, file).await;
//
//    Ok(())
//}
//
//pub async fn get_audio_information(id: &str) -> Result<models::SunoResponse> {
//    let base_url = "http://localhost:3000";
//    let url = format!("{}/api/get?ids={}", base_url, id);
//
//    let client = reqwest::Client::new();
//    let response = client.get(&url).send().await?;
//    let suno_response: Vec<models::SunoResponse> = response.json().await?;
//
//    suno_response
//        .into_iter()
//        .next()
//        .ok_or_else(|| anyhow!("No audio information found"))
//}
//
//pub async fn play_sound_instantly(
//    sink: &Sink,
//    file: BufReader<File>,
//) -> Result<()> {
//    match Decoder::new(BufReader::new(file)) {
//        Ok(v) => {
//            // This clear() seems to cause problems
//            // but it might be because we didn't pause enought before the append
//            // but that also would suck
//            // sink.clear();
//
//            println!("\tAppending Sound");
//            sink.append(v);
//
//            // If we sleep_until_end here,
//            // it blocks other commands in this ai_handler
//            // we might want to consider careful how to divide up these functions
//            // and share the proper handlers
//            // sink.sleep_until_end()
//        }
//        Err(e) => {
//            eprintln!("Error decoding sound file: {}", e);
//            return Err(anyhow!("Error decoding sound file: {}", e));
//        }
//    };
//
//    Ok(())
//}
//
//pub async fn generate_audio_by_prompt(
//    data: AudioGenerationData,
//) -> Result<serde_json::Value> {
//    let base_url = "http://localhost:3000";
//    let client = Client::new();
//    let url = format!("{}/api/generate", base_url);
//
//    // There must be a simpler way
//    let payload = serde_json::json!({
//        "prompt": data.prompt,
//        "make_instrumental": data.make_instrumental,
//        "wait_audio": data.wait_audio,
//    });
//    let response = client
//        .post(&url)
//        .json(&payload)
//        .header("Content-Type", "application/json")
//        .send()
//        .await?;
//    let raw_json = response.text().await?;
//    let tmp_file_path =
//        format!("tmp/suno_responses/{}.json", chrono::Utc::now().timestamp());
//    tokio::fs::write(&tmp_file_path, &raw_json).await?;
//    println!("Raw JSON saved to: {}", tmp_file_path);
//    Ok(serde_json::from_str::<serde_json::Value>(&raw_json)?)
//}
//
//pub async fn download_and_play(
//    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
//    tx: &broadcast::Sender<Event>,
//    user_name: String,
//    id: &String,
//) -> Result<()> {
//    let id = id.clone();
//    let tx = tx.clone();
//    let _twitch_client = twitch_client.clone();
//
//    thread::spawn(|| {
//        let rt = Runtime::new().unwrap();
//        rt.block_on(async move {
//            let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id.as_str());
//            loop {
//                println!(
//                    "{} | Attempting to Download song at: {}",
//                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
//                    cdn_url
//                );
//                let response = reqwest::get(&cdn_url).await.unwrap();
//
//                if response.status().is_success() {
//                    let _file = just_download(response, id.to_string()).await;
//
//                    let _info = format!(
//                        "@{}'s song {} added to the Queue. Begin needs to !skip to get to your position faster",
//                        user_name, id.to_string()
//                    );
//
//                    let _ =
//
//                        // We expect the song to playable at this time
//                        tx.send(Event::UserMessage(subd_types::UserMessage {
//                            user_name: "beginbot".to_string(),
//                            contents: format!("!play {}", id.to_string()),
//                            ..Default::default()
//                        }));
//
//                        // Should we try and save the ai_song_playlist
//
//                    break;
//                }
//
//                // Sleep for 5 seconds before trying again
//                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
//            }
//        })
//    });
//    return Ok(());
//}
//
//pub async fn parse_suno_response_download_and_play(
//    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
//    pool: &sqlx::PgPool,
//    tx: &broadcast::Sender<Event>,
//    json_response: serde_json::Value,
//    index: usize,
//    user_name: String,
//) -> Result<()> {
//    let id = json_response[index]["id"].as_str().unwrap();
//    let song_id = Uuid::parse_str(id).map_err(|e| anyhow!(e.to_string()))?;
//    let lyrics = &json_response[index]["lyric"].as_str().unwrap();
//    let title = &json_response[index]["title"].as_str().unwrap();
//    let prompt = &json_response[index]["prompt"].as_str().unwrap();
//    let tags = &json_response[index]["tags"].as_str().unwrap();
//    let audio_url = &json_response[index]["audio_url"].as_str().unwrap();
//    let gpt_description_prompt = &json_response[index]
//        ["gpt_description_prompt"]
//        .as_str()
//        .unwrap();
//    let created_at = sqlx::types::time::OffsetDateTime::now_utc();
//    let new_song = ai_playlist::models::ai_songs::Model {
//        song_id,
//        title: title.to_string(),
//        tags: tags.to_string(),
//        prompt: prompt.to_string(),
//        username: user_name.clone(),
//        audio_url: audio_url.to_string(),
//        lyric: Some(lyrics.to_string()),
//        gpt_description_prompt: gpt_description_prompt.to_string(),
//        last_updated: Some(created_at),
//        created_at: Some(created_at),
//    };
//    let saved_song = new_song.save(&pool).await?;
//    println!("{:?}", saved_song);
//
//    let _lyric_lines: Vec<&str> = lyrics.split('\n').collect();
//
//    let folder_path = format!("tmp/suno_responses/{}", id);
//    tokio::fs::create_dir_all(&folder_path).await?;
//
//    tokio::fs::write(
//        format!("tmp/suno_responses/{}.json", id),
//        &json_response.to_string(),
//    )
//    .await?;
//
//    // This needs to happen async and work properly
//    // for (_i, line) in lyric_lines.iter().enumerate() {
//    //     fal_handler::create_turbo_image_in_folder(
//    //         line.to_string(),
//    //         &folder_path,
//    //     )
//    //     .await?;
//    // }
//
//    download_and_play(twitch_client, tx, user_name, &id.to_string()).await
//}
//
//// We should return the file and have it played somewhere else
//pub async fn just_download(
//    response: reqwest::Response,
//    id: String,
//) -> Result<BufReader<File>> {
//    let file_name = format!("ai_songs/{}.mp3", id);
//    let mut file = tokio::fs::File::create(&file_name).await?;
//
//    let content = response.bytes().await?;
//    tokio::io::copy(&mut content.as_ref(), &mut file).await?;
//    println!("Downloaded audio to: {}", file_name);
//    let mp3 = match File::open(format!("{}", file_name)) {
//        Ok(v) => v,
//        Err(e) => {
//            eprintln!("Error opening sound file: {}", e);
//            return Err(anyhow!("Error opening sound file: {}", e));
//        }
//    };
//    let file = BufReader::new(mp3);
//
//    return Ok(file);
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use std::fs;
//
//    #[tokio::test]
//    #[ignore]
//    async fn test_parsing_json() {
//        let f = fs::read_to_string("tmp/raw_response_1725750380.json")
//            .expect("Failed to open file");
//        let suno_responses: Vec<models::SunoResponse> =
//            serde_json::from_str(&f).expect("Failed to parse JSON");
//
//        // let url = suno_responses[0].audio_url.as_str();
//        // tokio::io::copy(&mut content.as_ref(), &mut file).await.unwrap();
//        let id = &suno_responses[0].id;
//        println!("Suno URL: {}", suno_responses[0].audio_url.as_str());
//
//        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
//        let file_name = format!("ai_songs/{}.mp3", id);
//
//        let _response = reqwest::get(cdn_url).await.unwrap();
//        let mut _file = tokio::fs::File::create(file_name).await.unwrap();
//
//        // let mut content = Cursor::new(response.bytes().await.unwrap());
//        // std::io::copy(&mut content, &mut file).unwrap();
//
//        // assert!(!suno_responses.is_empty());
//        // assert_eq!(suno_responses[0].status, "completed");
//    }
//}

use anyhow::{anyhow, Result};
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

#[derive(Default, Debug, serde::Serialize)]
pub struct AudioGenerationData {
    pub prompt: String,
    pub make_instrumental: bool,
    pub wait_audio: bool,
}

/// Plays audio based on the provided song ID.
pub async fn play_audio(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    sink: &Sink,
    id: &str,
    user_name: &str,
) -> Result<()> {
    println!("\tQueuing {}", id);
    let info = format!("@{} added {} to Queue", user_name, id);
    send_message(&twitch_client, info).await?;

    let file_name = format!("ai_songs/{}.mp3", id);
    let mp3 = File::open(&file_name).map_err(|e| {
        anyhow!("Error opening sound file {}: {}", file_name, e)
    })?;
    let file = BufReader::new(mp3);
    println!("\tPlaying Audio {}", id);

    let uuid_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!("Invalid UUID {}: {}", id, e))?;

    println!("Adding to Playlist");
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
    ai_playlist::mark_song_as_played(pool, uuid_id).await?;

    play_sound_instantly(sink, file).await?;

    Ok(())
}

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

/// Plays sound instantly by appending it to the sink.
pub async fn play_sound_instantly(
    sink: &Sink,
    file: BufReader<File>,
) -> Result<()> {
    match Decoder::new(file) {
        Ok(decoder) => {
            println!("\tAppending Sound");
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

    tokio::spawn(async move {
        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
        loop {
            println!(
                "{} | Attempting to Download song at: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                cdn_url
            );
            match reqwest::get(&cdn_url).await {
                Ok(response) if response.status().is_success() => {
                    if let Err(e) = just_download(response, id.clone()).await {
                        eprintln!("Error downloading file: {}", e);
                    }

                    let info = format!(
                        "@{}'s song {} added to the Queue.",
                        user_name, id
                    );

                    if let Err(e) = send_message(&twitch_client, info).await {
                        eprintln!("Error sending message: {}", e);
                    }

                    let _ = tx.send(subd_types::Event::UserMessage(
                        subd_types::UserMessage {
                            user_name: "beginbot".to_string(),
                            contents: format!("!play {}", id),
                            ..Default::default()
                        },
                    ));

                    break;
                }
                Ok(_) => {
                    println!("Song not ready yet, retrying in 5 seconds...");
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

    let id = song_data
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'id' in song data"))?;

    let song_id = Uuid::parse_str(id)
        .map_err(|e| anyhow!("Invalid UUID {}: {}", id, e))?;

    let lyrics = song_data
        .get("lyric")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let title = song_data
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let prompt = song_data
        .get("prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tags = song_data.get("tags").and_then(|v| v.as_str()).unwrap_or("");
    let audio_url = song_data
        .get("audio_url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let gpt_description_prompt = song_data
        .get("gpt_description_prompt")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let created_at = sqlx::types::time::OffsetDateTime::now_utc();
    let new_song = ai_playlist::models::ai_songs::Model {
        song_id,
        title: title.to_string(),
        tags: tags.to_string(),
        prompt: prompt.to_string(),
        username: user_name.clone(),
        audio_url: audio_url.to_string(),
        lyric: Some(lyrics.to_string()),
        gpt_description_prompt: gpt_description_prompt.to_string(),
        last_updated: Some(created_at),
        created_at: Some(created_at),
    };
    new_song.save(&pool).await?;

    let folder_path = format!("tmp/suno_responses/{}", id);
    fs::create_dir_all(&folder_path).await?;

    fs::write(
        format!("tmp/suno_responses/{}.json", id),
        &json_response.to_string(),
    )
    .await?;

    download_and_play(twitch_client, tx, user_name, &id.to_string()).await
}

/// Downloads the audio file and saves it locally.
pub async fn just_download(
    response: reqwest::Response,
    id: String,
) -> Result<BufReader<File>> {
    let file_name = format!("ai_songs/{}.mp3", id);
    let mut file = fs::File::create(&file_name).await?;

    let content = response.bytes().await?;
    tokio::io::copy(&mut &content[..], &mut file).await?;
    println!("Downloaded audio to: {}", file_name);

    let mp3 = File::open(&file_name).map_err(|e| {
        anyhow!("Error opening sound file {}: {}", file_name, e)
    })?;
    let file = BufReader::new(mp3);

    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    #[ignore]
    async fn test_parsing_json() {
        let f = fs::read_to_string("tmp/raw_response.json")
            .expect("Failed to open file");
        let suno_responses: Vec<models::SunoResponse> =
            serde_json::from_str(&f).expect("Failed to parse JSON");

        assert!(!suno_responses.is_empty());
        assert_eq!(suno_responses[0].status, "completed");
    }
}
