use server::themesong::{play_themesong};
use subd_db::get_handle;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = get_handle().await;

    let user_id = 4;
    let _url = "https://www.youtube.com/watch?v=jOpzP33_USs";

    if true {
        // download_themesong(&mut db, &user_id, url, "00:01:03", "00:01:10").await?;
    }

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    play_themesong(&mut db, &user_id, &sink).await?;
    sink.sleep_until_end();

    Ok(())
}
