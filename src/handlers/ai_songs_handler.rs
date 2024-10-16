use ai_playlist;
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use events::EventHandler;
use rodio::Sink;
use sqlx::PgPool;
use std::time::Duration;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use tokio::time::interval;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use uuid::Uuid;

pub struct AISongsHandler {
    pub sink: Sink,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for AISongsHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let mut interval = interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if self.sink.empty() {
                        let _ = ai_playlist::mark_songs_as_stopped(&self.pool).await;
                        let next_song = ai_playlist::find_next_song_to_play(&self.pool).await;
                        if let Ok(song) = next_song {
                            let id = song.song_id.to_string();

                            // // We need to be able to toggle this
                            // let custom_prompt = format!("{} {}", song.title, song.lyric.unwrap_or_default());
                            // let _ = tokio::spawn(
                            //     subd_openai::ai_styles::generate_ai_css(id.clone(), "./static/styles.css", custom_prompt.clone(), None));
                            // let _ = tokio::spawn(
                            //     subd_openai::ai_styles::generate_ai_js(id.clone(), "./static/styles.js", custom_prompt.clone(), None));

                            if let Err(e) = subd_suno::play_audio(&self.pool, &self.sink, &id).await {
                                eprint!("Error playing Audio: {}", e);
                                let _ = ai_playlist::mark_song_as_played(&self.pool, song.song_id).await;

                            }
                        }
                    }
                }
                result = rx.recv() => {
                    let event = result?;
                    let msg = match event {
                        Event::UserMessage(msg) => msg,
                        _ => continue,
                    };

                    let splitmsg: Vec<String> =
                        msg.contents.split_whitespace().map(String::from).collect();

                    if let Err(err) = handle_requests(
                        &self.sink,
                        &self.twitch_client,
                        &self.pool,
                        &splitmsg,
                        &msg,
                    )
                    .await
                    {
                        eprintln!("Error in AISongsHandler: {err}");
                        continue;
                    }
                }
            }
        }
    }
}

/// Determines if the user is an admin (beginbot or beginbotbot)
fn is_admin(msg: &UserMessage) -> bool {
    msg.user_name == "beginbot" || msg.user_name == "beginbotbot"
}

/// Handles incoming requests based on commands
async fn handle_requests(
    sink: &Sink,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    splitmsg: &[String],
    msg: &UserMessage,
) -> Result<()> {
    // Extract the command from the split message
    let command = splitmsg.first().map(|s| s.as_str()).unwrap_or("");

    match command {
        // Commands accessible to all users
        "!info" => {
            handle_info_command(twitch_client, pool, splitmsg, msg).await?;
        }

        // Commands requiring admin privileges
        "!reverb"
        | "!queue"
        | "!play"
        | "!play_fake_song"
        | "!pause"
        | "!random_song"
        | "!last_song"
        | "!unpause"
        | "!skip"
        | "!stop"
        | "!nightcore"
        | "!doom"
        | "!normal"
        | "!speedup"
        | "!slowdown"
        | "!up"
        | "!down"
        | "!coding_volume"
        | "!quiet"
        | "!party_volume"
        | "!delete_song"
        | "!banger"
        | "!random_instrumental"
        | "!instrumental_jam" => {
            if !is_admin(msg) {
                return Ok(());
            }

            match command {
                "!delete" => {
                    let id = match splitmsg.get(1) {
                        Some(id) => id,
                        None => {
                            println!("{}", "No ID provided for delete.".red());
                            return Ok(());
                        }
                    };
                    let uuid_id = Uuid::parse_str(id)?;
                    ai_playlist::delete_song(pool, uuid_id).await?
                }
                "!reverb" => {
                    handle_reverb_command(
                        twitch_client,
                        pool,
                        sink,
                        splitmsg,
                        msg,
                    )
                    .await?
                }
                "!queue" => handle_queue_command(pool, splitmsg).await?,
                "!last_song" => {
                    handle_last_song_command(twitch_client, pool).await?
                }

                // !random 5 to add random songs to the queue
                "!random_song" => {
                    let index = splitmsg
                        .get(1)
                        .unwrap_or(&"1".to_string())
                        .parse::<usize>()
                        .unwrap_or(1);
                    handle_random_song_command(twitch_client, pool, index)
                        .await?
                }
                "!play_fake_song" => {
                    handle_fake_play_command(
                        twitch_client,
                        pool,
                        sink,
                        splitmsg,
                        msg,
                    )
                    .await?
                }

                // This does actually work queueing the songs
                "!banger" => {
                    println!("Time for a Banger!");
                    // let song = ai_songs_vote::get_random_high_rated_song(&pool)
                    let song =
                        ai_songs_vote::get_random_high_rated_recent_song(pool)
                            .await?;
                    let message = format!("!queue {}", song.song_id);
                    let _ = send_message(twitch_client, message).await;
                    let message =
                        format!("!Added Song to Queue - {}", song.title);
                    let _ = send_message(twitch_client, message).await;
                    return Ok(());
                }

                // TODO: Confirm this works
                "!random_instrumental" | "!instrumental_jam" => {
                    println!("Random Instrumental Time!");
                    let song =
                        ai_playlist::models::find_random_instrumental(pool)
                            .await?;

                    let message = format!("!queue {}", song.song_id);
                    let _ = send_message(twitch_client, message).await;
                    let message = format!(
                        "@{} Added Song to Queue - {} | {}",
                        song.username, song.title, song.tags
                    );
                    let _ = send_message(twitch_client, message).await;
                    return Ok(());
                }

                "!play" => {
                    handle_play_command(
                        twitch_client,
                        pool,
                        sink,
                        splitmsg,
                        msg,
                    )
                    .await?
                }
                "!pause" | "!unpause" | "!skip" | "!stop" => {
                    handle_playback_control(command, sink).await?
                }
                "!nightcore" | "!doom" | "!normal" | "!speedup"
                | "!slowdown" => handle_speed_control(command, sink).await?,
                "!up" | "!down" | "!coding_volume" | "!quiet"
                | "!party_volume" => {
                    handle_volume_control(command, sink).await?
                }
                _ => {}
            }
        }

        // Unknown or unhandled commands
        _ => {}
    }

    Ok(())
}

/// Handles the "!info" command to display current song info
async fn handle_info_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    splitmsg: &[String],
    _msg: &UserMessage,
) -> Result<()> {
    if let Some(id) = splitmsg.get(1) {
        let res = subd_suno::get_audio_information(id).await?;
        println!("Suno Response: {:?}", res);
    } else {
        let song = match ai_playlist::get_current_song(pool).await {
            Ok(song) => song,
            Err(_) => {
                let _ = send_message(
                    twitch_client,
                    // What should we do so it's not read
                    "!No current song playing".to_string(),
                )
                .await;
                return Ok(());
            }
        };
        let res =
            subd_suno::get_audio_information(&format!("{}", song.song_id))
                .await?;
        println!("\tSuno Response: {:?}", res);
        let score =
            match ai_songs_vote::get_average_score(pool, song.song_id).await {
                Ok(ranking) => ranking.avg_score.to_string(),
                Err(_) => "No Votes for Song".to_string(),
            };

        let message = format!(
            "@{}'s Song is Currently playing - {} | {} | {} | AVG Score: {}",
            song.username, song.title, song.tags, song.song_id, score
        );
        let _ = send_message(twitch_client, message).await;
    }
    Ok(())
}

/// Handles the "!reverb" command to play audio with reverb
async fn handle_reverb_command(
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &PgPool,
    sink: &Sink,
    splitmsg: &[String],
    msg: &UserMessage,
) -> Result<()> {
    let id = match splitmsg.get(1) {
        Some(id) => id,
        None => {
            println!("No ID provided for reverb.");
            return Ok(());
        }
    };

    println!("Queuing with Reverb: {}", id);
    subd_suno::add_to_playlist_and_play_audio(pool, sink, id, &msg.user_name)
        .await?;
    Ok(())
}

async fn handle_random_song_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
    amount: usize,
) -> Result<()> {
    // We could also make sure we don't pull duplicates
    for _ in 0..amount {
        let song = ai_playlist::find_random_song(pool).await?;
        let message = format!("!queue {}", song.song_id);
        let _ = send_message(twitch_client, message).await;
        let message = format!(
            "@{} Added Song to Queue - {} | {}",
            song.username, song.title, song.tags
        );
        let _ = send_message(twitch_client, message).await;
    }

    Ok(())
}

async fn handle_last_song_command(
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &PgPool,
) -> Result<()> {
    let uuid = ai_playlist::find_last_played_song(pool).await?;
    let message = format!("!play {}", uuid);
    let _ = send_message(twitch_client, message).await;
    Ok(())
}

/// Handles the "!queue" command to add a song to the playlist
async fn handle_queue_command(
    pool: &PgPool,
    splitmsg: &[String],
) -> Result<()> {
    let id = match splitmsg.get(1) {
        Some(id) => id,
        None => return Ok(()),
    };

    let uuid_id = Uuid::parse_str(id)?;
    ai_playlist::add_song_to_playlist(pool, uuid_id).await?;
    Ok(())
}

/// Handles the "!play" command to play a song immediately
async fn handle_fake_play_command(
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &PgPool,
    sink: &Sink,
    splitmsg: &[String],
    msg: &UserMessage,
) -> Result<()> {
    let id = match splitmsg.get(1) {
        Some(id) => id,
        None => return Ok(()),
    };

    println!("Attempting to use UUID: {}", id);

    //
    let title = match splitmsg.get(2..) {
        Some(title_parts) => title_parts.join(" "),
        None => return Ok(()),
    };

    let created_at = sqlx::types::time::OffsetDateTime::now_utc();

    // At this point it should be downloaded I think???
    let song_id = Uuid::parse_str(id)?;
    let new_song = ai_playlist::models::ai_songs::Model {
        song_id,
        title,
        tags: "".to_string(),
        prompt: "".to_string(),
        username: "beginbot".to_string(),
        audio_url: "".to_string(),
        lyric: None,
        gpt_description_prompt: "".to_string(),
        last_updated: Some(created_at),
        created_at: Some(created_at),
        downloaded: true,
    };

    // Save the song if it doesn't already exist
    let _ = new_song.save(pool).await;

    // Play the audio
    subd_suno::add_to_playlist_and_play_audio(pool, sink, id, &msg.user_name)
        .await?;
    Ok(())
}
/// Handles the "!play" command to play a song immediately
async fn handle_play_command(
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    pool: &PgPool,
    sink: &Sink,
    splitmsg: &[String],
    msg: &UserMessage,
) -> Result<()> {
    let id = match splitmsg.get(1) {
        Some(id) => id,
        None => return Ok(()),
    };

    // TODO: consider if we do want to do updates
    // Do we need to do this everytime????
    // Fetch audio information
    // let audio_info = subd_suno::get_audio_information(id).await?;
    // let created_at = sqlx::types::time::OffsetDateTime::now_utc();

    // We should only be able to play songs that exist in the DB
    // let song = ai_playlist::find_song_by_id(pool, id).await?;
    // We don't know if the song is downloaded here!!!!
    // At this point it should be downloaded I think???
    // let song_id = Uuid::parse_str(&audio_info.id)?;
    // let new_song = ai_playlist::models::ai_songs::Model {
    //     song_id,
    //     title: audio_info.title,
    //     tags: audio_info.metadata.tags,
    //     prompt: audio_info.metadata.prompt,
    //     username: msg.user_name.clone(),
    //     audio_url: audio_info.audio_url,
    //     lyric: audio_info.lyric,
    //     gpt_description_prompt: audio_info.metadata.gpt_description_prompt,
    //     last_updated: Some(created_at),
    //     created_at: Some(created_at),
    //     // We need to not update this!!!!!
    //     downloaded: true,
    // };

    // Save the song if it doesn't already exist
    // let _ = new_song.save(pool).await;

    // This is all we really need
    // Play the audio
    subd_suno::add_to_playlist_and_play_audio(pool, sink, id, &msg.user_name)
        .await?;
    Ok(())
}

/// Handles playback control commands like pause, unpause, skip, and stop
async fn handle_playback_control(command: &str, sink: &Sink) -> Result<()> {
    match command {
        "!pause" => {
            println!("\t{}", "Pausing playback.".yellow());
            sink.pause();
        }
        "!unpause" => {
            println!("\t{}", "Resuming playback.".yellow());
            sink.play();
        }
        "!skip" => {
            println!("\t{}", "Skipping current track.".yellow());
            sink.skip_one();
            sink.play();
        }
        "!stop" => {
            println!("\t{}", "Stopping playback.".yellow());
            sink.stop();
        }
        _ => {}
    }
    Ok(())
}

/// Handles speed control commands like nightcore, doom, speedup, and slowdown
async fn handle_speed_control(command: &str, sink: &Sink) -> Result<()> {
    match command {
        "!nightcore" => {
            println!("\t{}", "Setting speed to Nightcore (1.5x).".yellow());
            sink.set_speed(1.5);
        }
        "!doom" => {
            println!("\t{}", "Setting speed to Doom (0.5x).".yellow());
            sink.set_speed(0.5);
        }
        "!normal" => {
            println!("\t{}", "Resetting speed to normal (1.0x).".yellow());
            sink.set_speed(1.0);
        }
        "!speedup" => {
            println!("\t{}", "Increasing playback speed.".yellow());
            sink.set_speed(sink.speed() * 1.25);
        }
        "!slowdown" => {
            println!("\t{}", "Decreasing playback speed.".yellow());
            sink.set_speed(sink.speed() * 0.75);
        }
        _ => {}
    }
    Ok(())
}

/// Handles volume control commands like up, down, quiet, and party volume
async fn handle_volume_control(command: &str, sink: &Sink) -> Result<()> {
    match command {
        "!up" => {
            println!("\t{}", "Increasing volume.".yellow());
            sink.set_volume(sink.volume() * 1.20);
        }
        "!down" => {
            println!("\t{}", "Decreasing volume.".yellow());
            sink.set_volume(sink.volume() * 0.80);
        }
        "!coding_volume" | "!quiet" => {
            println!("\t{}", "Setting volume for coding (0.1).".yellow());
            sink.set_volume(0.1);
        }
        "!party_volume" => {
            println!("\t{}", "Setting volume to party level (1.0).".yellow());
            sink.set_volume(1.0);
        }
        _ => {}
    }
    Ok(())
}
