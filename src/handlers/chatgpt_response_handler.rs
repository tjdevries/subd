use anyhow::Result;
use std::thread;
use std::time;
use std::time::Duration;
use async_trait::async_trait;
use events::EventHandler;
use subd_types::Event;

use twitch_chat::send_message;

use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncReadExt;

use tokio::sync::broadcast;
use twitch_irc::{TwitchIRCClient, SecureTCPTransport, login::StaticLoginCredentials};

pub struct ChatGPTResponse {
    pub twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
impl EventHandler for ChatGPTResponse {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let response_file = "/home/begin/code/BeginGPT/tmp/current/chatgpt_response.txt";
        let metadata = fs::metadata(response_file).await.unwrap();
        let mut last_modified = metadata.modified().unwrap();
        
        loop {
            let metadata = fs::metadata(response_file).await.unwrap();
            let current_modified = metadata.modified().unwrap();
            println!("Current Modified: {:?}", current_modified);
            
            if current_modified > last_modified {
                let mut file = fs::File::open(response_file).await.unwrap();
                
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents).await;

                println!("File changed: {:?}", contents);
                let chunk_size = 500;
                
                for chunk in contents.chars().collect::<Vec<char>>().chunks(chunk_size) {
                    // Process each chunk
                    let chunk_str: String = chunk.iter().collect();
                    println!("{}", chunk_str);
                    send_message(&self.twitch_client, chunk_str).await?;
                    // Add your logic here
                }

                // How do I truncate the string to only 500
                println!("New Current Modified: {:?}", current_modified);
                last_modified = current_modified;
                
            }
            let sleep_time = time::Duration::from_millis(1000);
            thread::sleep(sleep_time);
        }
    }
}
