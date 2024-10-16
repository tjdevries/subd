use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use std::thread;
use std::time;
use subd_types::Event;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct ChatGPTResponse {
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for ChatGPTResponse {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        // TODO: This needs to be updated and re-considered
        // TODO: this should be somewhere else
        let response_file =
            "/home/begin/code/BeginGPT/tmp/current/chatgpt_response.txt";
        let metadata = fs::metadata(response_file)
            .await
            .expect("Failed to read metadata");
        let mut last_modified = metadata
            .modified()
            .expect("Failed to get last modified time");

        loop {
            let metadata = match fs::metadata(response_file).await {
                Ok(meta) => meta,
                Err(e) => {
                    println!("Error reading metadata: {:?}", e);
                    continue;
                }
            };
            let current_modified = match metadata.modified() {
                Ok(modified) => modified,
                Err(e) => {
                    println!("Error getting modified time: {:?}", e);
                    continue;
                }
            };

            if current_modified > last_modified {
                let mut file = match fs::File::open(response_file).await {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Error opening response file: {}", e);
                        continue;
                    }
                };

                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents).await;

                let voice_text = contents.clone();
                let _ = tx.send(Event::ElevenLabsRequest(
                    subd_types::ElevenLabsRequest {
                        source: Some("begin".to_string()),
                        voice_text,
                        message: contents,
                        username: "beginbot".to_string(),

                        ..Default::default()
                    },
                ));

                println!("New Current Modified: {:?}", current_modified);
                last_modified = current_modified;
            }
            let sleep_time = time::Duration::from_millis(1000);
            thread::sleep(sleep_time);
        }
    }
}
