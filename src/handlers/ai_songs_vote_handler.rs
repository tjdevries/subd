use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use sqlx::PgPool;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsVoteHandler {
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for AISongsVoteHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // THEORY: We don't know if this is an explicit OBS message at this stage
            match handle_telephone_requests(
                &tx,
                &self.obs_client,
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

pub async fn handle_telephone_requests(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let command = splitmsg[0].as_str();

    match command {
        "!top_songs" => {
            //
            let songs = ai_songs_vote::get_top_songs(pool, 5).await?;
            // let mut stats = "Top Songs: ".to_string();
            for (index, song) in songs.iter().enumerate() {
                let rank_msg = &format!(
                    "!Song #{} | {}: {:.2}\n",
                    index + 1,
                    song.title,
                    song.avg_score
                );
                let _ = send_message(twitch_client, rank_msg).await;
            }
            Ok(())
        }

        "!hate" => {
            let image_base_name = splitmsg[1].as_str();
            let _ = handle_love_or_hate(
                pool,
                twitch_client,
                msg,
                image_base_name,
                ImageVoteOption::Hate,
            )
            .await;
            Ok(())
        }

        "!love" => {
            let image_base_name = splitmsg[1].as_str();
            let _ = handle_love_or_hate(
                pool,
                twitch_client,
                msg,
                image_base_name,
                ImageVoteOption::Love,
            )
            .await;
            Ok(())
        }

        "!vote" => {
            let score = splitmsg
                .get(1)
                .ok_or_else(|| anyhow!("No score provided"))?
                .parse::<f64>()?;

            if !(0.0..=10.0).contains(&score) {
                let _ = send_message(
                    twitch_client,
                    "You must vote between 0.0 and 10.0",
                )
                .await;
                return Ok(());
            }

            println!("Voting for {}", score);
            ai_songs_vote::vote_for_current_song_with_score(
                pool,
                msg.user_id.into(),
                score,
            )
            .await
        }

        _ => Ok(()),
    }
}

enum ImageVoteOption {
    Love,
    Hate,
}

async fn handle_love_or_hate(
    pool: &sqlx::PgPool,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    msg: UserMessage,
    image_base_name: &str,
    image_vote_option: ImageVoteOption,
) -> Result<()> {
    let image_name = format!("{}.png", image_base_name);
    let current_song = ai_playlist::get_current_song(pool).await?;
    let image_path =
        format!("./tmp/music_videos/{}/{}", current_song.song_id, image_name);

    if std::path::Path::new(&image_path).exists() {
        match image_vote_option {
            ImageVoteOption::Love => {
                let _love_vote =
                    ai_playlist::models::image_votes::Model::love_image(
                        pool,
                        msg.user_id.into(),
                        current_song.song_id,
                        &image_name,
                    )
                    .await?;
            }
            ImageVoteOption::Hate => {
                let _love_vote =
                    ai_playlist::models::image_votes::Model::hate_image(
                        pool,
                        msg.user_id.into(),
                        current_song.song_id,
                        &image_name,
                    )
                    .await?;
            }
        }
    } else {
        let _ = send_message(
            twitch_client,
            &format!("Image {} not found for the current song.", image_name),
        )
        .await;
    }
    Ok(())
}
