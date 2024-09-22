use ai_playlist::models::ai_songs;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rodio::Sink;
use sqlx::PgPool;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use uuid::Uuid;

pub struct AISongsHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for AISongsHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            if self.sink.empty() {
                println!("It's empty!");
                println!("Marking all Songs as stopped");
                let _ = ai_playlist::mark_songs_as_stopped(&self.pool).await;
            } else {
                println!("It's not empty!");
            }

            let event = rx.recv().await?;

            // I could check the sink right here
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // THEORY: We don't know if this is an explicit OBS message at this stage
            match handle_requests(
                &tx,
                &self.obs_client,
                &self.sink,
                &self.twitch_client,
                &self.pool,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
    }
}

pub async fn handle_requests(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    sink: &Sink,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();

    match command {
        "!info" => {
            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => {
                    let song = ai_playlist::get_current_song(pool).await?;
                    let msg = format!(
                        "Current Song: {} by {}",
                        song.title, song.username
                    );
                    // If the message doesn't send we don't care...or we do
                    let _ = send_message(twitch_client, msg).await;
                    return Ok(());
                }
            };

            let res = subd_suno::get_audio_information(id).await?;
            println!("Suno Response: {:?}", res);
            // We query for info
            return Ok(());
        }

        "!reverb" => {
            if _not_beginbot {
                return Ok(());
            }

            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => {
                    println!("No ID Found to reverb to add reverb");
                    return Ok(());
                }
            };

            // TODO: relook at reverb
            // sink.try_seek() and you might need the position before you move it
            // add_source(song, reverb) ->sink.skip_one(); sink.seek(sink.get_pos())
            println!("\tQueuing w/ Reverb {}", id);
            // let reverb = true;
            return subd_suno::play_audio(
                &twitch_client,
                pool,
                &sink,
                id,
                &msg.user_name,
            )
            .await;
        }
        // ================= //
        // Playback Controls //
        // ================= //
        "!queue" => {
            if _not_beginbot {
                return Ok(());
            }

            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => return Ok(()),
            };

            // let reverb = false;
            // let _audio_info = get_audio_information(id).await?;

            let uuid_id = uuid::Uuid::parse_str(id)?;
            ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
            return Ok(());
        }

        "!play" => {
            if _not_beginbot {
                return Ok(());
            }

            let id = match splitmsg.get(1) {
                Some(id) => id.as_str(),
                None => return Ok(()),
            };

            // ============================================

            // The song needs to exist here!!!
            // let reverb = false;
            let audio_info = subd_suno::get_audio_information(id).await?;
            let created_at = sqlx::types::time::OffsetDateTime::now_utc();

            let song_id = Uuid::parse_str(&audio_info.id)?;
            let new_song = ai_songs::Model {
                song_id,
                title: audio_info.title,
                tags: audio_info.metadata.tags,
                prompt: audio_info.metadata.prompt,
                username: msg.user_name.clone(),
                audio_url: audio_info.audio_url,
                lyric: audio_info.lyric,
                gpt_description_prompt: audio_info
                    .metadata
                    .gpt_description_prompt,
                last_updated: Some(created_at),
                created_at: Some(created_at),
            };

            // HA WE ARE TRYING
            // If we already have the song, we don't need to crash
            let _saved_song = new_song.save(&pool).await;

            let _ = subd_suno::play_audio(
                &twitch_client,
                pool,
                &sink,
                id,
                &msg.user_name,
            )
            .await;
            return Ok(());
        }

        "!pause" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to !pause");
            sink.pause();
            println!("\tDone !pause");
            return Ok(());
        }

        "!unpause" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tTrying to Pause!");
            sink.play();
            println!("\tDone Pausing");
            return Ok(());
        }

        "!skip" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to Skip!");
            sink.skip_one();
            sink.play();
            println!("\tDone Attempting to Skip!");
            return Ok(());
        }

        "!stop" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("\tAttempting to Stop!");
            sink.stop();
            println!("\tDone Attempting to Stop!");
            return Ok(());
        }

        // =============== //
        // Speed Controls //
        // =============== //
        "!nightcore" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\nNightcore Time");
            sink.set_speed(1.5);
            return Ok(());
        }

        "!doom" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\nDoom Time");
            sink.set_speed(0.5);
            return Ok(());
        }

        "!normal" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tNormal Time");
            sink.set_speed(1.0);
            return Ok(());
        }

        "!speedup" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tSpeeding up!");
            sink.set_speed(sink.speed() * 1.25);
            return Ok(());
        }

        "!slowdown" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tSlowin down!");
            sink.set_speed(sink.speed() * 0.75);
            return Ok(());
        }

        // =============== //
        // Volume Controls //
        // =============== //
        "!up" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tTurning it Up!");
            sink.set_volume(sink.volume() * 1.20);
            return Ok(());
        }

        "!down" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tTurning it Down!");
            sink.set_volume(sink.volume() * 0.80);
            return Ok(());
        }

        "!coding_volume" | "!quiet" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tTurning it down so we can code!");
            sink.set_volume(0.1);
            return Ok(());
        }

        "!party_volume" => {
            if _not_beginbot {
                return Ok(());
            }
            println!("\tParty Volume");
            sink.set_volume(1.0);
            return Ok(());
        }

        // Reverb
        _ => {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_parsing_json() {
        let f = fs::read_to_string("tmp/raw_response_1725750380.json")
            .expect("Failed to open file");
        let suno_responses: Vec<subd_suno::SunoResponse> =
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
