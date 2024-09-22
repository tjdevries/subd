use anyhow::anyhow;
use anyhow::Result;
use rodio::Decoder;
use rodio::Sink;
use std::fs::File;
use std::io::BufReader;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub mod models;

pub async fn play_audio(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    sink: &Sink,
    id: &str,
    user_name: &str,
) -> Result<()> {
    println!("\tQueuing {}", id);
    let info = format!("@{} added {} to Queue", user_name, id);
    let _ = send_message(&twitch_client, info).await;

    let file_name = format!("ai_songs/{}.mp3", id);
    let mp3 = match File::open(&file_name) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening sound file: {}", e);
            return Ok(());
        }
    };
    let file = BufReader::new(mp3);
    println!("\tPlaying Audio {}", id);

    let uuid_id = uuid::Uuid::parse_str(id)?;

    println!("Adding to Playlist");
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
    ai_playlist::mark_song_as_played(pool, uuid_id).await?;

    let _ = play_sound_instantly(sink, file).await;

    Ok(())
}

pub async fn get_audio_information(id: &str) -> Result<models::SunoResponse> {
    let base_url = "http://localhost:3000";
    let url = format!("{}/api/get?ids={}", base_url, id);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let suno_response: Vec<models::SunoResponse> = response.json().await?;

    suno_response
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No audio information found"))
}

async fn play_sound_instantly(
    sink: &Sink,
    file: BufReader<File>,
) -> Result<()> {
    match Decoder::new(BufReader::new(file)) {
        Ok(v) => {
            // This clear() seems to cause problems
            // but it might be because we didn't pause enought before the append
            // but that also would suck
            // sink.clear();

            println!("\tAppending Sound");
            sink.append(v);

            // If we sleep_until_end here,
            // it blocks other commands in this ai_handler
            // we might want to consider careful how to divide up these functions
            // and share the proper handlers
            // sink.sleep_until_end()
        }
        Err(e) => {
            eprintln!("Error decoding sound file: {}", e);
            return Err(anyhow!("Error decoding sound file: {}", e));
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    #[ignore]
    async fn test_parsing_json() {
        let f = fs::read_to_string("tmp/raw_response_1725750380.json")
            .expect("Failed to open file");
        let suno_responses: Vec<models::SunoResponse> =
            serde_json::from_str(&f).expect("Failed to parse JSON");

        // let url = suno_responses[0].audio_url.as_str();
        // tokio::io::copy(&mut content.as_ref(), &mut file).await.unwrap();
        let id = &suno_responses[0].id;
        println!("Suno URL: {}", suno_responses[0].audio_url.as_str());

        let cdn_url = format!("https://cdn1.suno.ai/{}.mp3", id);
        let file_name = format!("ai_songs/{}.mp3", id);

        let _response = reqwest::get(cdn_url).await.unwrap();
        let mut _file = tokio::fs::File::create(file_name).await.unwrap();

        // let mut content = Cursor::new(response.bytes().await.unwrap());
        // std::io::copy(&mut content, &mut file).unwrap();

        // assert!(!suno_responses.is_empty());
        // assert_eq!(suno_responses[0].status, "completed");
    }
}
