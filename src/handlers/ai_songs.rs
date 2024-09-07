use async_trait::async_trait;
use tokio::sync::broadcast;
use anyhow::anyhow;
use anyhow::Result;
use obws::Client as OBSClient;
use reqwest::Client;
use events::EventHandler;
use sqlx::PgPool;
use rodio::Sink;
use subd_types::{Event, UserMessage};
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};

pub struct AISongsHandler {
    pub sink: Sink,
    pub obs_client: OBSClient,
    pub pool: PgPool,
    pub twitch_client: TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}



// impl EventHandler for AiScenesHandler {
//     async fn handle(
//         self: Box<Self>,
//         _tx: broadcast::Sender<Event>,
//         mut rx: broadcast::Receiver<Event>,
//     ) -> Result<()> {

        // self: Box<Self>,
        // sink: Sink,
        // pool: PgPool,
        // twitch_client:
        //     TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,

#[async_trait]
impl EventHandler for AISongsHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            let event = rx.recv().await?;
            let msg = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = msg
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // THEORY: We don't know if this is an explicit OBS message at this stage
            match handle_requests(
                &tx,
                &self.obs_client,
                &self.sink,
                &self.twitch_client,
                &self.pool,
                splitmsg,
                msg,
            )
            .await
            {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {err}");
                    continue;
                }
            }
        }
        
        Ok(())
    }
}


#[derive(Default, Debug)]
struct AudioGenerationData {
    prompt: String,
    make_instrumental: bool,
    wait_audio: bool,
}

async fn generate_audio_by_prompt(data: AudioGenerationData) -> Result<serde_json::Value> {
    let base_url = "http://localhost:3000";
    let client = Client::new();
    let url = format!("{}/api/generate", base_url);

    // There must be a simpler way
    let payload = serde_json::json!({
        "prompt": data.prompt,
        "make_instrumental": data.make_instrumental,
        "wait_audio": data.wait_audio,
    });
    let response = client.post(&url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let json_response = response.json().await?;
    Ok(json_response)
}

pub async fn handle_requests(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    sink: &Sink,
    _twitch_client: &TwitchIRCClient<
        SecureTCPTransport,
        StaticLoginCredentials,
    >,
    _pool: &sqlx::PgPool,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";
    let command = splitmsg[0].as_str();
    let prompt = splitmsg[1..].to_vec().join(" ");
    
    match command {
        "!song" => {
            if _not_beginbot {
                return Ok(());
            }

            println!("It's Song time!");
            let data = AudioGenerationData {
                prompt: prompt,
                make_instrumental: false,
                wait_audio: true,
            };
            let res = generate_audio_by_prompt(data).await;
            // We have some text
            return Ok(());
        }

        _ => {
            return Ok(());
        }
    };
}
