use std::io::{BufReader, Cursor};

use anyhow::Result;
use async_trait::async_trait;
use psl::Psl;
use reqwest::Url;
use sqlx::PgPool;
use subd_types::{Event, ThemesongDownload, ThemesongPlay, UserID, UserRoles};
use tokio::{fs::File, io::AsyncReadExt, sync::broadcast};
use tracing::info;

use crate::audio;

const THEMESONG_LOCATION: &str = "/tmp/themesong";

pub async fn play_themesong_for_today(
    conn: &mut PgPool,
    user_id: &UserID,
    sink: &rodio::OutputStreamHandle,
) -> Result<()> {
    if has_played_themesong_today(conn, user_id).await? {
        return Ok(());
    }

    if play_themesong(conn, user_id, sink).await? {
        mark_themesong_played(conn, user_id).await?;
    }

    Ok(())
}

pub async fn delete_themesong(pool: &PgPool, display_name: &str) -> Result<()> {
    let _display_name = display_name.replace("@", "").to_lowercase();
    // let user_id =
    //     subd_db::get_user_from_twitch_user_name(conn, display_name.as_str())
    //         .await?;

    let user_id = uuid::Uuid::new_v4();
    sqlx::query!("DELETE FROM user_theme_songs WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn mark_themesong_played(pool: &PgPool, user_id: &UserID) -> Result<()> {
    // Insert that we've played their theme song
    sqlx::query!(
        "INSERT INTO USER_THEME_SONG_HISTORY (user_id) VALUES ($1)",
        user_id.0
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_themesong_unplayed(
    pool: &PgPool,
    user_id: &UserID,
) -> Result<()> {
    // Insert that we've played their theme song
    sqlx::query!(
        "DELETE FROM USER_THEME_SONG_HISTORY WHERE user_id = ($1) AND date(played_at) = date(CURRENT_TIMESTAMP)",
        user_id.0
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn has_played_themesong_today(
    pool: &PgPool,
    user_id: &UserID,
) -> Result<bool> {
    let played_count = sqlx::query!(
        r#"
        SELECT count(*) AS RESULT
          FROM user_theme_song_history
          WHERE date(played_at) = date('now')
            AND user_id = $1;
        "#,
        user_id.0
    )
    .fetch_one(pool)
    .await?;

    Ok(played_count.result.unwrap() > 0)
}

#[tracing::instrument(skip(conn))]
pub async fn download_themesong(
    conn: &PgPool,
    user_id: &UserID,
    user_name: &str,
    url: &str,
    start: &str,
    end: &str,
) -> Result<()> {
    // TODO: Use temp_file and/or temp_dir for this
    // TODO: Use ytextract to make sure that the video is < 1 hour or something like that
    // TODO: Also could probably use --max-filesize as well or in place of ytextract

    let url = validate_themesong(url)?;
    let duration = match user_name {
        "conni2461" => 30.0,
        _ => 10.0,
    };
    validate_duration(start, end, duration)?;

    let location =
        THEMESONG_LOCATION.to_string() + user_id.0.to_string().as_str();

    info!("downloading");

    // TODO: .extra_arg("-f bestaudio")
    youtube_dl::YoutubeDl::new(url)
        .youtube_dl_path("yt-dlp")
        .extract_audio(true)
        .download(true)
        // Don't allow downloading playlists
        .extra_arg("--no-playlist")
        // Don't continue a paused download, always restart
        .extra_arg("--no-continue")
        .extra_arg("--downloader")
        .extra_arg("ffmpeg")
        .extra_arg("--downloader-args")
        .extra_arg(format!("-ss {} -to {}", start, end))
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("-o")
        .extra_arg(location.clone() + ".%(ext)s")
        .run()?;

    info!("successfully downloaded yt clip");

    let mut contents = vec![];

    let mut f = File::open(location.clone() + ".mp3").await?;
    f.read_to_end(&mut contents).await?;

    // Delete the previous theme song
    let user_id = user_id.0;
    sqlx::query!("DELETE FROM USER_THEME_SONGS WHERE user_id = $1", user_id)
        .execute(conn)
        .await?;

    // Insert the new theme song
    sqlx::query!(
        "INSERT INTO USER_THEME_SONGS (user_id, song) VALUES ($1, $2)",
        user_id,
        contents
    )
    .execute(conn)
    .await?;

    // And now delete the file
    std::fs::remove_file(location.clone() + ".mp3")?;

    Ok(())
}

pub fn can_user_access_themesong(user_roles: &UserRoles) -> bool {
    user_roles.is_github_sponsor()
        || user_roles.is_twitch_mod()
        || user_roles.is_twitch_sub()
        || user_roles.is_twitch_vip()
        || user_roles.is_twitch_founder()
}

// TODO: We should probably not copy & paste this like this
pub async fn should_play_themesong(
    conn: &PgPool,
    user_id: &UserID,
    user_roles: &UserRoles,
) -> Result<bool> {
    if has_played_themesong_today(conn, user_id).await? {
        return Ok(false);
    }

    if !can_user_access_themesong(&user_roles) {
        return Ok(false);
    }

    let themesong = sqlx::query!(
        "SELECT user_id FROM user_theme_songs WHERE user_id = $1",
        user_id.0
    )
    .fetch_optional(conn)
    .await?;

    match themesong {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

// Play a themesong. Does not wait for sink to complete playing
pub async fn play_themesong(
    pool: &PgPool,
    user_id: &UserID,
    sink: &rodio::OutputStreamHandle,
) -> Result<bool> {
    println!("playing themesong... {:?}", user_id);

    let themesong = sqlx::query!(
        "SELECT song FROM user_theme_songs WHERE user_id = $1",
        user_id.0
    )
    .fetch_optional(pool)
    .await?;

    let themesong = match themesong {
        Some(themesong) => themesong,
        None => {
            println!("theme_song: No themesong available for: {:?}", user_id);
            return Ok(false);
        }
    };

    println!("got themesong");

    // let rodioer =
    //     rodio::Decoder::new(BufReader::new(Cursor::new(themesong.song)))
    //         .unwrap();
    // TODO: I would like to turn this off after the sink finishes playing, but I don't know how to
    // do that yet, this probably wouldn't work with queued themesongs (for example)
    // rodioer.total_duration();

    println!("appending...");

    let sink = sink
        .play_once(BufReader::new(Cursor::new(themesong.song)))
        .expect("to play a song");

    sink.detach();

    // sink.append(rodioer);
    // sink.play();
    // sink.sleep_until_end();
    // sink.sleep_until_end();

    println!("... done");

    // TODO: Look into using these!
    // self.sink.volume()
    // self.sink.set_volume()
    // self.sink.len()

    Ok(true)
}

pub fn validate_themesong(themesong_url: &str) -> Result<String> {
    let mut themesong_url = themesong_url.to_string();
    let mut parsed = Url::parse(themesong_url.as_str())?;

    let domain = match parsed.domain() {
        Some(domain) => domain,
        None => return Err(anyhow::anyhow!("no domain provided")),
    };

    let domain = (psl::List {})
        .domain(domain.as_bytes())
        .ok_or(anyhow::anyhow!("invalid domain"))?;

    if !(domain == "youtube.com"
        || domain == "youtu.be"
        || domain == "twitch.tv"
        || domain == "twitter.com"
        || domain == "beginworld.website-us-east-1.linodeobjects.com"
        || domain == "media1.vocaroo.com")
    {
        return Err(anyhow::anyhow!(
            "invalid host. must be youtube.com or clips.twitch.tv -> {:?}",
            domain
        ));
    }

    if domain == "youtube.com" {
        let cloned = parsed.clone();
        let v = cloned.query_pairs().find(|(name, _)| name == "v");
        if let Some((name, value)) = v {
            parsed.query_pairs_mut().clear().append_pair(&name, &value);
            themesong_url = parsed.to_string();
        } else {
            return Err(anyhow::anyhow!("Missing v for YouTube link"));
        }
    } else if domain == "youtu.be" {
        if parsed.query_pairs().count() > 0 {
            parsed.query_pairs_mut().clear();
        }

        themesong_url = parsed.to_string();
    }

    // TODO: If ppl are being stinkers, we may have to check the length
    // and information about the video before allowing the download
    Ok(themesong_url)
}

pub fn validate_duration(start: &str, end: &str, maxtime: f64) -> Result<()> {
    // 01:10, 01:23

    let (start_minutes, start_seconds) = start
        .split_once(":")
        .ok_or(anyhow::anyhow!("Must be single : split str"))?;

    let (end_minutes, end_seconds) = end
        .split_once(":")
        .ok_or(anyhow::anyhow!("Must be single : split str"))?;

    // TODO: Support ms for ppl
    let start =
        start_minutes.parse::<f64>()? * 60.0 + start_seconds.parse::<f64>()?;
    let end =
        end_minutes.parse::<f64>()? * 60.0 + end_seconds.parse::<f64>()?;

    if end - start <= 0.0 {
        Err(anyhow::anyhow!("End must be after start"))
    } else if end - start > maxtime {
        Err(anyhow::anyhow!("Too long. Choose shorter clip"))
    } else {
        Ok(())
    }
}

pub struct ThemesongPlayer {
    pool: PgPool,
    // sink: rodio::Sink,
    handle: rodio::OutputStreamHandle,
}

impl ThemesongPlayer {
    pub fn new(pool: PgPool) -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();

        // TODO: How to now leak this... it's ok though, it just gets called once
        Box::leak(Box::new(stream));

        ThemesongPlayer { pool, handle }
    }
}

#[async_trait]
impl events::EventHandler for ThemesongPlayer {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            match event {
                Event::ThemesongPlay(ThemesongPlay::Start {
                    user_id,
                    display_name,
                }) => {
                    println!("=> Playing themesong: {}", display_name);

                    play_themesong_for_today(
                        &mut self.pool,
                        &user_id,
                        &self.handle,
                    )
                    .await?;
                }
                Event::UserMessage(msg) => {
                    if should_play_themesong(
                        &mut self.pool,
                        &msg.user_id,
                        &msg.roles,
                    )
                    .await?
                    {
                        println!("  Sending themesong play event...");
                        tx.send(Event::ThemesongPlay(ThemesongPlay::Start {
                            user_id: msg.user_id,
                            display_name: msg.user_name,
                        }))?;
                    }
                }

                _ => continue,
            };
        }
    }
}

pub struct ThemesongListener {}

impl ThemesongListener {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl events::EventHandler for ThemesongListener {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            if msg.contents.starts_with("!themesong") {
                tx.send(Event::ThemesongDownload(
                    ThemesongDownload::Request { msg },
                ))?;
            }
        }
    }
}

pub struct ThemesongDownloader {
    pool: sqlx::PgPool,
    users: user_service::Service,
}

impl ThemesongDownloader {
    pub fn new(pool: sqlx::PgPool, users: user_service::Service) -> Self {
        Self { pool, users }
    }
}

#[async_trait]
impl events::EventHandler for ThemesongDownloader {
    async fn handle(
        mut self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::ThemesongDownload(ThemesongDownload::Request {
                    msg,
                }) => msg,
                _ => continue,
            };

            let user_id = msg.user_id;
            let user_roles = self
                .users
                .get_roles(&user_id)
                .await?
                .ok_or(anyhow::anyhow!("empty user_roles"))?;

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            if splitmsg.len() == 1 {
                tx.send(Event::RequestTwitchMessage(
                    "format: !themesong <url> mm:ss mm:ss".to_string(),
                ))?;

                tx.send(ThemesongDownload::format(msg.user_name))?;
                continue;
            } else if splitmsg.len() != 4 {
                tx.send(Event::RequestTwitchMessage(
                    "Incorrect themesong format. Required: !themesong <url> mm:ss mm:ss".to_string()
                ))?;

                tx.send(ThemesongDownload::finish(msg.user_name, false))?;
                continue;
            }

            if can_user_access_themesong(&user_roles) {
                // Notify that we are starting a download
                tx.send(ThemesongDownload::start(msg.user_name.clone()))?;

                match download_themesong(
                    &self.pool,
                    &user_id,
                    msg.user_name.as_str(),
                    splitmsg[1].as_str(),
                    splitmsg[2].as_str(),
                    splitmsg[3].as_str(),
                )
                .await
                {
                    Ok(_) => {
                        info!("Successfully downloaded themesong");

                        tx.send(ThemesongDownload::finish(
                            msg.user_name,
                            true,
                        ))?;

                        continue;
                    }
                    Err(err) => {
                        tx.send(Event::RequestTwitchMessage(format!(
                            "Failed to download: {:?}",
                            err
                        )))?;

                        tx.send(Event::ThemesongDownload(
                            ThemesongDownload::Finish {
                                display_name: msg.user_name.clone(),
                                success: false,
                            },
                        ))?;

                        continue;
                    }
                };
            } else {
                tx.send(Event::RequestTwitchMessage(
                    "You must be a GH Sponsor or sub/mod/VIP to do this"
                        .to_string(),
                ))?;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accepts_youtube_dot_com() {
        assert!(validate_themesong(
            "https://www.youtube.com/watch?v=SkypZuY6ZvA"
        )
        .is_ok())
    }

    #[test]
    fn accepts_twitch_dot_tv() {
        assert!(validate_themesong(
            "https://www.twitch.tv/teej_dv/clip/EsteemedBrightMartenAsianGlow-5Fgv6QEy8zT3_tOq"
        )
        .is_ok())
    }

    #[test]
    fn accepts_begin_world() {
        assert!(validate_themesong(
            "https://beginworld.website-us-east-1.linodeobjects.com/commands/stupac62.html"
        )
        .is_ok())
    }

    #[test]
    fn accepts_youtu_be_links_with_no_v() {
        assert_eq!(
            validate_themesong("https://youtu.be/QMVIJhC9Veg").unwrap(),
            "https://youtu.be/QMVIJhC9Veg".to_string()
        );

        assert_eq!(
            validate_themesong("https://youtu.be/QMVIJhC9Veg?anything=asdf")
                .unwrap(),
            "https://youtu.be/QMVIJhC9Veg?".to_string()
        );
    }

    #[test]
    fn acceps_playlists_but_strips_url() {
        assert_eq!(
            validate_themesong(
                "https://www.youtube.com/watch?v=XZtL7PsJAoc&list=RDXZtL7PsJAoc&start_radio=1"
            )
            .unwrap(),
            "https://www.youtube.com/watch?v=XZtL7PsJAoc".to_string()
        )
    }

    #[test]
    fn accepts_simple_timestamps() {
        assert!(validate_duration("00:05", "00:10", 10.0).is_ok());
        assert!(validate_duration("01:58", "02:05", 10.0).is_ok());
        assert!(validate_duration("01:58.231", "02:05.09", 10.).is_ok());
        assert!(validate_duration("01:58", "02:05", 3.).is_err());
        assert!(validate_duration("00:05", "00:00", 10.).is_err());
        assert!(validate_duration("00:05", "00:50", 10.).is_err());
    }
}
