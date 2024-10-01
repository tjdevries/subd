use crate::{redemptions, twitch_rewards};
use ai_scenes_coordinator;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::Extension, http::StatusCode, response::IntoResponse,
    routing::post, Json, Router, Server,
};
use colored::Colorize;
use events::EventHandler;
use obws::Client as OBSClient;
use openai::chat;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::{net::SocketAddr, sync::Arc};
use subd_openai;
use subd_twitch::rewards::{self, RewardManager};
use subd_types::Event;
use tokio::sync::broadcast;
use twitch_irc::{
    login::StaticLoginCredentials, SecureTCPTransport, TwitchIRCClient,
};
use twitch_stream_state;

pub struct TwitchEventSubHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
    pub twitch_client:
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventSubRoot {
    pub subscription: Subscription,
    pub event: Option<SubEvent>,
    pub challenge: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscription {
    id: String,
    status: String,
    #[serde(rename = "type")]
    type_field: String,
    version: String,
    condition: Condition,
    transport: Transport,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Condition {
    broadcaster_user_id: String,
    reward_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Reward {
    title: String,
    cost: i32,
    id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transport {
    method: String,
    callback: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubEvent {
    id: Uuid,
    user_id: String,
    user_login: String,
    user_name: String,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    title: Option<String>,
    tier: Option<String>,
    is_gift: Option<bool>,
    reward: Option<Reward>,
    user_input: Option<String>,
}

#[async_trait]
impl EventHandler for TwitchEventSubHandler {
    async fn handle(
        self: Box<Self>,
        tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let obs_client = Arc::new(self.obs_client);
        let pool = Arc::new(self.pool);
        let reward_manager = rewards::build_reward_manager().await?;
        let reward_manager = Arc::new(reward_manager);

        println!("{}", "Kicking off a new reward router!".yellow());

        let app = Router::new()
            .route("/eventsub", post(post_request::<reqwest::Client>))
            .layer(Extension(obs_client))
            .layer(Extension(pool))
            .layer(Extension(reward_manager))
            .layer(Extension(tx))
            .layer(Extension(self.twitch_client));

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
            Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        Ok(())
    }
}

async fn post_request<'a, C: twitch_api::HttpClient>(
    Json(eventsub_body): Json<EventSubRoot>,
    Extension(obs_client): Extension<Arc<OBSClient>>,
    Extension(pool): Extension<Arc<sqlx::PgPool>>,
    Extension(reward_manager): Extension<Arc<RewardManager<'a, C>>>,
    Extension(tx): Extension<broadcast::Sender<Event>>,
    Extension(_twitch_client): Extension<
        TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>,
    >,
) -> impl IntoResponse {
    if let Some(challenge) = eventsub_body.challenge {
        println!("We got a challenge!");
        return (StatusCode::OK, challenge);
    }

    if let Some(event) = eventsub_body.event {
        match eventsub_body.subscription.type_field.as_str() {
            "channel.channel_points_custom_reward_redemption.add" => {
                if let Err(e) = handle_channel_rewards_request(
                    tx,
                    pool,
                    &obs_client,
                    reward_manager,
                    event,
                )
                .await
                {
                    eprintln!("Error handling reward request: {:?}", e);
                }
            }
            _ => println!("Unhandled event type"),
        }
    }

    (StatusCode::OK, "".to_string())
}

async fn process_ai_scene(
    tx: broadcast::Sender<Event>,
    scene: &ai_scenes_coordinator::models::AIScene,
    user_input: String,
    enable_dalle: bool,
    enable_stable_diffusion: bool,
) -> Result<()> {
    println!("{} {}", "Asking Chat GPT:".green(), user_input);
    let content = ask_chat_gpt(&user_input, &scene.base_prompt).await?;
    println!("\n{} {}", "Chat GPT response: ".green(), content);

    let dalle_prompt = if enable_dalle || enable_stable_diffusion {
        Some(format!("{} {}", scene.base_dalle_prompt, user_input))
    } else {
        None
    };

    println!("Triggering Scene: {}", scene.voice);
    tx.send(Event::AiScenesRequest(subd_types::AiScenesRequest {
        voice: Some(scene.voice.clone()),
        message: content.clone(),
        voice_text: content,
        music_bg: Some(scene.music_bg.clone()),
        prompt: dalle_prompt,
        ..Default::default()
    }))?;

    Ok(())
}

// =========================================================================

async fn ask_chat_gpt(user_input: &str, base_prompt: &str) -> Result<String> {
    let response = subd_openai::ask_chat_gpt(
        user_input.to_string(),
        base_prompt.to_string(),
    )
    .await
    .map_err(|e| {
        eprintln!("Error occurred: {:?}", e);
        anyhow!("Error response")
    })?;

    let content = response.content.ok_or_else(|| anyhow!("No content"))?;

    match content {
        chat::ChatCompletionContent::Message(message) => {
            Ok(message.unwrap_or_default())
        }
        chat::ChatCompletionContent::VisionMessage(messages) => messages
            .iter()
            .find_map(|msg| {
                if let chat::VisionMessage::Text { text, .. } = msg {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("No text content found")),
    }
}

// REWARDS ==============================================================

async fn handle_channel_rewards_request<'a, C: twitch_api::HttpClient>(
    tx: broadcast::Sender<Event>,
    pool: Arc<sqlx::PgPool>,
    _obs_client: &OBSClient,
    reward_manager: Arc<RewardManager<'a, C>>,
    event: SubEvent,
) -> Result<()> {
    let state = twitch_stream_state::get_twitch_state(&pool).await?;
    let ai_scenes_map = ai_scenes_coordinator::load_ai_scenes()?;

    let reward = match event.reward {
        Some(r) => r,
        None => return Ok(()),
    };

    let command = reward.title.clone();
    println!("{} {}", "Processing AI Scene: ".cyan(), command.green());

    let user_input = event.user_input.unwrap_or_default();

    find_or_save_redemption(
        pool.clone(),
        reward_manager,
        event.id,
        command.clone(),
        reward.id,
        reward.cost,
        event.user_name.clone(),
        user_input.clone(),
    )
    .await?;

    if command == "Set Theme" {
        println!("Setting the Theme: {}", &user_input);
        twitch_stream_state::set_ai_background_theme(&pool, &user_input)
            .await?;
        return Ok(());
    }

    if let Some(scene) = ai_scenes_map.get(&command) {
        process_ai_scene(
            tx,
            scene,
            user_input,
            state.dalle_mode,
            state.enable_stable_diffusion,
        )
        .await?;
    } else {
        println!("Scene not found for reward title")
    }

    Ok(())
}

async fn find_or_save_redemption<'a, C: twitch_api::HttpClient>(
    pool: Arc<sqlx::PgPool>,
    reward_manager: Arc<RewardManager<'a, C>>,
    id: Uuid,
    command: String,
    reward_id: Uuid,
    reward_cost: i32,
    user_name: String,
    user_input: String,
) -> Result<()> {
    if redemptions::find_redemption_by_twitch_id(&pool, id)
        .await
        .is_ok()
    {
        println!("\nRedemption already exists: {}\n", command);
        return Ok(());
    }

    // Saving new redemption: Command: Ask Melkey a Question ID: 9e3e2aa3-7135-4de3-8604-1b9a81f1ddb7
    //
    // Error handling reward request: error returned from database: duplicate key value violates unique constraint "redemptions_reward_id_key"
    //
    // Caused by:
    //     duplicate key value violates unique constraint "redemptions_reward_id_key"

    println!(
        "\nSaving new redemption: Command: {} | ID: {} | Reward ID: {} \n",
        command, id, reward_id
    );
    redemptions::save_redemptions(
        &pool,
        command.clone(),
        reward_cost,
        user_name,
        id,
        reward_id,
        user_input,
    )
    .await?;

    adjust_reward_cost(&pool, &reward_manager, reward_id, reward_cost).await?;
    Ok(())
}

async fn adjust_reward_cost<'a, C: twitch_api::HttpClient>(
    pool: &Arc<sqlx::PgPool>,
    reward_manager: &Arc<RewardManager<'a, C>>,
    reward_id: Uuid,
    reward_cost: i32,
) -> Result<()> {
    let increase_mult = 1.5;
    let new_cost = (reward_cost as f32 * increase_mult).round() as usize;

    println!("Updating Reward: {} - New Cost: {}", reward_id, new_cost);
    reward_manager
        .update_reward(reward_id.to_string(), new_cost)
        .await?;

    twitch_rewards::update_cost_by_id(pool, reward_id, new_cost as i32).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_uuid_parsing() {
        let uuid_str = "ba11ad0f-dad5-c001-c001-700bac001e57";
        assert!(Uuid::parse_str(uuid_str).is_ok());
    }
}
