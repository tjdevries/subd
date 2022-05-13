use serde_json::json;
use std::net::TcpListener;

use tokio;
use tungstenite::{accept, Message};

// enum WebsockMessages {
//     Display(String),
//     ShowImg((String, Coordinates)),
// }

/// A WebSocket echo server
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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

                // websocket
                //     .write_message(Message::Text(
                //         json!({
                //             "variant": "other",
                //             "data": 5,
                //         })
                //         .to_string(),
                //     ))
                //     .unwrap();

                count = count + 1;
                if count == 3 {
                    websocket
                        .write_message(Message::Text(
                            json!({
                                "variant": "show-image",
                                "data": "./ty_quadstew.png",
                            })
                            .to_string(),
                        ))
                        .unwrap();
                }
            }
        }
        // });
    }

    Ok(())
}
