use serde_json::json;
use sqlx::{Connection, SqliteConnection};
use std::net::TcpListener;

use tokio;
use tungstenite::{accept, Message};

/// A WebSocket echo server
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut conn =
        SqliteConnection::connect(&std::env::var("DATABASE_URL").expect("gotta have dat db"))
            .await?;

    let mut count = 0;

    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    println!("Succesfully bound!");
    for stream in server.incoming() {
        // spawn(move || {
        let mut websocket = accept(stream.unwrap()).unwrap();
        loop {
            let msg = websocket.read_message().unwrap();

            // We do not want to send back ping/pong messages.
            if msg.is_binary() || msg.is_text() {
                println!("{msg}");

                add_id(&mut conn, msg.to_text()?).await?;
                let resp = msg.to_text()?.to_string();
                websocket
                    .write_message(Message::Text(
                        json!({
                            "variant": "display",
                            "data": resp,
                        })
                        .to_string(),
                    ))
                    .unwrap();

                websocket
                    .write_message(Message::Text(
                        json!({
                            "variant": "other",
                            "data": 5,
                        })
                        .to_string(),
                    ))
                    .unwrap();

                count = count + 1;
                if count == 3 {
                    websocket
                        .write_message(Message::Text("Wow, the third message".to_string()))
                        .unwrap();
                }
            }
        }
        // });
    }

    Ok(())
}

async fn add_id(conn: &mut SqliteConnection, message: &str) -> anyhow::Result<i64> {
    // let mut conn = pool.acquire().await?;

    // Insert the task, then obtain the ID of this row
    let id = sqlx::query!(
        r#"
INSERT INTO NYX_LUL ( message )
VALUES ( ?1 )
        "#,
        message
    )
    .execute(conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}
