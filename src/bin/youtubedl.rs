use subd_db::get_db_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _db = get_db_pool().await;

    let _user_id = 4;
    let _url = "https://www.youtube.com/watch?v=jopzp33_uss";

    if true {
        // download_themesong(&mut db, &user_id, url, "00:01:03", "00:01:10").await?;
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    // play_themesong(&mut db, &user_id, &sink).await?;
    sink.sleep_until_end();

    Ok(())
}
