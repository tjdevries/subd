use std::io::{BufReader, Cursor};

use subd_db::get_db_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let songpath = "/home/tjdevries/Downloads/teej_dv..wav";
    // let contents = fs::read(songpath).await?;

    let db = get_db_pool().await;
    // sqlx::query!(
    //     "INSERT INTO user_theme_songs (user_id, song) VALUES (117, ?1)",
    //     contents
    // )
    // .execute(&mut db)
    // .await?;
    let user_id = uuid::Uuid::nil();
    let result = sqlx::query!(
        "SELECT song FROM user_theme_songs WHERE user_id = $1",
        user_id
    )
    .fetch_one(&db)
    .await?;

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let x = Cursor::new(result.song);
    sink.append(rodio::Decoder::new(BufReader::new(x)).unwrap());

    sink.sleep_until_end();

    Ok(())
}
