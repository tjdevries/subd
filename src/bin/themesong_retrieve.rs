use std::{
    io::{BufReader, Cursor},
    time::Duration,
};

use rodio::Source;
use server::commands::ThemeSong;
use subd_db::get_handle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = get_handle().await;

    let user_id = "123";
    let theme = ThemeSong {
        slug: "5lLclBfKj48".to_string(),
        start: 65,
        duration: 6,
    };

    // Get a Client for making request
    let client = ytextract::Client::new();

    // Get information about the Video identified by the id "nI2e-J6fsuk".
    // let video = client.video(slug.parse()?).await?;
    // video.duration();
    // println!("Title: {}", video.title());

    let existing_song = sqlx::query!(
        "SELECT song, start_seconds, duration FROM user_theme_songs WHERE user_id = ?1",
        user_id
    )
    .fetch_optional(&mut db)
    .await?;
    println!("... Attempting to download theme song");
    if let None = existing_song {
        let streams = client.streams(theme.slug.parse()?).await?;
        for stream in streams {
            match stream {
                ytextract::Stream::Audio(stream) => {
                    println!("MIME TYPE: {:?}", stream.mime_type());
                    if !stream.mime_type().starts_with("audio/mp4") {
                        continue;
                    }

                    // let range_value = format!(
                    //     "bytes={}-{}",
                    //     bitrate * start_second / 8,
                    //     bitrate * end_second / 8
                    // );

                    println!("Getting request");
                    let url = stream.url();
                    let client = reqwest::Client::builder()
                    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.0) Gecko/20100101 Firefox/102.0")
                    .build()?;

                    // let result = client.get(url).header("Range", range_value).send().await?;
                    let result = client.get(url).send().await?;
                    // dbg!(&result);
                    let content = result.bytes().await?.to_vec();
                    println!("Got request: #content {:?}", content.len());

                    sqlx::query!(
                        "INSERT INTO user_theme_songs (user_id, song, start_seconds, duration) VALUES (?1, ?2, ?3, ?4)",
                        user_id,
                        content,
                        theme.start,
                        theme.duration,
                    )
                    .execute(&mut db)
                    .await?;

                    break;
                }
                ytextract::Stream::Video(_) => {}
            }
        }
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let content = sqlx::query!(
        "SELECT song, start_seconds, duration FROM user_theme_songs WHERE user_id = ?1",
        user_id
    )
    .fetch_one(&mut db)
    .await?;

    let x = Cursor::new(content.song);
    let rodioer = rodio::Decoder::new(BufReader::new(x)).unwrap();
    let rodioer = rodioer.skip_duration(Duration::from_secs(content.start_seconds as u64));
    let rodioer = rodioer.take_duration(Duration::from_secs(content.duration as u64));
    sink.append(rodioer);

    println!("Trying to play");
    sink.sleep_until_end();

    Ok(())
}
