use std::io::{BufReader, Cursor};

use anyhow::Result;
use psl::Psl;
use reqwest::Url;
use sqlx::SqliteConnection;
use subd_types::UserID;
use tokio::{fs::File, io::AsyncReadExt};
use twitch_irc::message::PrivmsgMessage;

const THEMESONG_LOCATION: &str = "/tmp/themesong";

pub async fn play_themesong_for_today(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    sink: &rodio::Sink,
) -> Result<()> {
    if has_played_themesong_today(conn, user_id).await? {
        return Ok(());
    }

    if play_themesong(conn, user_id, sink).await? {
        mark_themesong_played(conn, user_id).await?;
    }

    Ok(())
}

pub async fn reset_themesong(conn: &mut SqliteConnection, display_name: &str) -> Result<()> {
    let display_name = display_name.replace("@", "").to_lowercase();
    let user_id = subd_db::get_user_from_twitch_user_name(conn, display_name.as_str()).await?;

    sqlx::query!("DELETE FROM user_theme_songs WHERE user_id = ?1", user_id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

async fn mark_themesong_played(conn: &mut SqliteConnection, user_id: &UserID) -> Result<()> {
    // Insert that we've played their theme song
    sqlx::query!(
        "INSERT INTO USER_THEME_SONG_HISTORY (user_id) VALUES (?1)",
        user_id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn has_played_themesong_today(
    conn: &mut SqliteConnection,
    user_id: &UserID,
) -> Result<bool> {
    let played_count = sqlx::query!(
        r#"
            SELECT count(*) as result
            FROM USER_THEME_SONG_HISTORY
            WHERE date(played_at) = date('now') AND user_id = ?1;
        "#,
        user_id
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(played_count.result > 0)
}

pub async fn download_themesong(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    url: &str,
    start: &str,
    end: &str,
) -> Result<()> {
    validate_themesong(url)?;
    validate_duration(start, end, 10.)?;

    println!("youtube_dl: Downloading == {:?}", url);
    youtube_dl::YoutubeDl::new(url)
        .youtube_dl_path("yt-dlp")
        .extract_audio(true)
        .download(true)
        // .extra_arg("-f bestaudio")
        .extra_arg("--downloader")
        .extra_arg("ffmpeg")
        .extra_arg("--downloader-args")
        .extra_arg(format!("-ss {} -to {}", start, end))
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("-o")
        .extra_arg(THEMESONG_LOCATION.to_string() + ".%(ext)s")
        .run()?;
    println!("  Done!");

    let mut contents = vec![];

    let mut f = File::open(THEMESONG_LOCATION.to_string() + ".mp3").await?;
    f.read_to_end(&mut contents).await?;

    // Delete the previous theme song
    sqlx::query!("DELETE FROM USER_THEME_SONGS WHERE user_id = ?1", user_id)
        .execute(&mut *conn)
        .await?;

    // Insert the new theme song
    sqlx::query!(
        "INSERT INTO USER_THEME_SONGS (user_id, song) VALUES (?1, ?2)",
        user_id,
        contents
    )
    .execute(&mut *conn)
    .await?;

    // And now delete the file
    std::fs::remove_file(THEMESONG_LOCATION.to_string() + ".mp3")?;

    Ok(())
}

// TODO: We should probably not copy & paste this like this
pub async fn should_play_themesong(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<bool> {
    if has_played_themesong_today(conn, user_id).await? {
        return Ok(false);
    }

    // Only mods, founders & subs can do this
    if !msg.badges.iter().any(|badge| {
        badge.name == "moderator" || badge.name == "founder" || badge.name == "subscriber"
    }) {
        return Ok(false);
    }

    let themesong = sqlx::query!(
        "SELECT song FROM user_theme_songs WHERE user_id = ?1",
        user_id
    )
    .fetch_optional(&mut *conn)
    .await?;

    match themesong {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

// Play a themesong. Does not wait for sink to complete playing
pub async fn play_themesong(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    sink: &rodio::Sink,
) -> Result<bool> {
    let themesong = sqlx::query!(
        "SELECT song FROM user_theme_songs WHERE user_id = ?1",
        user_id
    )
    .fetch_optional(&mut *conn)
    .await?;

    let themesong = match themesong {
        Some(themesong) => themesong,
        None => {
            println!("theme_song: No themesong available for: {:?}", user_id);
            return Ok(false);
        }
    };

    let rodioer = rodio::Decoder::new(BufReader::new(Cursor::new(themesong.song))).unwrap();
    sink.append(rodioer);

    Ok(true)
}

pub fn validate_themesong(themesong_url: &str) -> Result<()> {
    let parsed = Url::parse(themesong_url)?;

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
        || domain == "beginworld.website-us-east-1.linodeobjects.com")
    {
        return Err(anyhow::anyhow!(
            "invalid host. must be youtube.com or clips.twitch.tv"
        ));
    }

    // TODO: If ppl are being stinkers, we may have to check the length
    // and information about the video before allowing the download

    Ok(())
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
    let start = start_minutes.parse::<f64>()? * 60.0 + start_seconds.parse::<f64>()?;
    let end = end_minutes.parse::<f64>()? * 60.0 + end_seconds.parse::<f64>()?;

    if end - start <= 0.0 {
        Err(anyhow::anyhow!("End must be after start"))
    } else if end - start > maxtime {
        Err(anyhow::anyhow!("Too long. Choose shorter clip"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accepts_youtube_dot_com() {
        assert!(validate_themesong("https://www.youtube.com/watch?v=SkypZuY6ZvA").is_ok())
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
    fn accepts_simple_timestamps() {
        assert!(validate_duration("00:05", "00:10", 10.0).is_ok());
        assert!(validate_duration("01:58", "02:05", 10.0).is_ok());
        assert!(validate_duration("01:58.231", "02:05.09", 10.).is_ok());
        assert!(validate_duration("01:58", "02:05", 3.).is_err());
        assert!(validate_duration("00:05", "00:00", 10.).is_err());
        assert!(validate_duration("00:05", "00:50", 10.).is_err());
    }
}
