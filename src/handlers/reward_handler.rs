use crate::ai_scene;
use crate::twitch_rewards;
use anyhow::Result;
use async_trait::async_trait;
use events::EventHandler;
use obws::Client as OBSClient;
use rand::Rng;
use std::fs;
use subd_twitch::rewards;
use subd_types::Event;
use subd_types::UserMessage;
use tokio;
use tokio::sync::broadcast;
use twitch_chat::client::send_message;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use uuid::Uuid;

pub struct RewardHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for RewardHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        mut rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let reward_manager = rewards::build_reward_manager().await?;

        loop {
            let event = rx.recv().await?;
            let request = match event {
                Event::UserMessage(msg) => msg,
                _ => continue,
            };

            let splitmsg = request
                .contents
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let command = splitmsg[0].as_str();
            if let Err(e) = route_messages(
                &tx,
                &self.obs_client,
                &self.twitch_client,
                &self.pool,
                &reward_manager,
                command,
                request,
            )
            .await
            {
                eprintln!("Erroring routing Reward Message: {}", e);
            }
        }
    }
}

async fn route_messages<C: twitch_api::HttpClient>(
    _tx: &broadcast::Sender<Event>,
    _obs_client: &OBSClient,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    pool: &sqlx::PgPool,
    reward_manager: &rewards::RewardManager<'_, C>,
    command: &str,
    msg: UserMessage,
) -> Result<()> {
    // let is_mod = msg.roles.is_twitch_mod();
    // let is_vip = msg.roles.is_twitch_vip();
    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    match command {
        "!flash_sale" => {
            if not_beginbot {
                return Ok(());
            }
            let ai_scenes = current_ai_scenes()?;
            let title = find_random_ai_scene_title(ai_scenes)?;
            let _ =
                flash_sale(title, reward_manager, pool, twitch_client).await?;
        }

        "!bootstrap_rewards" => {
            if not_beginbot {
                return Ok(());
            }
            let ai_scenes = current_ai_scenes()?;
            for scene in ai_scenes.scenes {
                let cost = scene.cost * 10;
                let res = reward_manager
                    .create_reward(&scene.reward_title, cost)
                    .await?;

                let reward_id = res.as_str();
                let reward_id = Uuid::parse_str(reward_id)?;

                let _ = twitch_rewards::save_twitch_rewards(
                    &pool.clone(),
                    scene.reward_title,
                    cost,
                    reward_id,
                    true,
                )
                .await;
            }

            return Ok(());
        }
        _ => {}
    }
    Ok(())
}

pub async fn flash_sale<C: twitch_api::HttpClient>(
    title: String,
    reward_manager: &rewards::RewardManager<'_, C>,
    pool: &sqlx::PgPool,
    twitch_client: &TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
) -> Result<()> {
    // This goes in subd-twitch
    // If we don't have a reward for that Thang
    let reward_res =
        twitch_rewards::find_by_title(&pool, title.to_string()).await?;
    let flash_cost = 100;
    let _ = reward_manager
        .update_reward(reward_res.twitch_id.to_string(), flash_cost)
        .await;

    let update = twitch_rewards::update_cost(
        &pool,
        reward_res.title.to_string(),
        flash_cost as i32,
    )
    .await?;

    println!("Update: {:?}", update);
    let msg = format!(
        "Flash Sale! {} - New Low Price! {}",
        reward_res.title, flash_cost
    );
    let _ = send_message(&twitch_client, msg).await;

    Ok(())
}

fn find_random_ai_scene_title(ai_scenes: ai_scene::AIScenes) -> Result<String> {
    let random = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..ai_scenes.scenes.len())
    };
    let random_scene = &ai_scenes.scenes[random];
    Ok(random_scene.reward_title.clone())
}

// TODO: Don't hardcode this
fn current_ai_scenes() -> Result<ai_scene::AIScenes> {
    let file_path = "/home/begin/code/subd/data/AIScenes.json";
    let contents = fs::read_to_string(file_path)?;
    let ai_scenes: ai_scene::AIScenes =
        serde_json::from_str(&contents.clone())?;
    Ok(ai_scenes)
}
