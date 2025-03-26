use crate::{redemptions, twitch_rewards};
use ai_scenes_coordinator;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::FromRef, extract::State, http::StatusCode, routing::post, Json,
    Router,
};
use colored::Colorize;
use events::EventHandler;
use obws::Client as OBSClient;
use openai::chat;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::sync::Arc;
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

#[derive(Clone)]
struct AppState<'a, C: twitch_api::HttpClient> {
    obs_client: Arc<OBSClient>,
    pool: Arc<sqlx::PgPool>,
    tx: broadcast::Sender<Event>,
    reward_manager: Arc<RewardManager<'a, C>>,
    // RewardManager<&'static, twitch_api::HttpClient<Error = anyhow::Error>>,
    // reward_manager: Box<dyn RewardManager<'a, twitch_api::HttpClient>>,
}

impl<C: twitch_api::HttpClient> FromRef<AppState<'_, C>>
    for broadcast::Sender<Event>
{
    fn from_ref(app_state: &AppState<C>) -> broadcast::Sender<Event> {
        app_state.tx.clone()
    }
}

impl<C: twitch_api::HttpClient> FromRef<AppState<'_, C>> for Arc<OBSClient> {
    fn from_ref(app_state: &AppState<C>) -> Arc<OBSClient> {
        app_state.obs_client.clone()
    }
}

impl<C: twitch_api::HttpClient> FromRef<AppState<'_, C>> for Arc<sqlx::PgPool> {
    fn from_ref(app_state: &AppState<C>) -> Arc<sqlx::PgPool> {
        app_state.pool.clone()
    }
}

impl<'a, C: twitch_api::HttpClient> FromRef<AppState<'a, C>>
    for Arc<RewardManager<'a, C>>
{
    fn from_ref(app_state: &AppState<'a, C>) -> Arc<RewardManager<'a, C>> {
        app_state.reward_manager.clone()
    }
}

#[async_trait]
impl EventHandler for TwitchEventSubHandler {
    // #[axum::debug_handler]
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

        let state = AppState {
            obs_client,
            pool,
            tx,
            reward_manager,
        };

        let app = Router::new()
            .route("/eventsub", post(simple_post_request))
            .with_state(state);

        tokio::spawn(async move {
            let listener =
                tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        Ok(())
    }
}

async fn simple_post_request<'a, C: twitch_api::HttpClient>(
    State(obs_client): State<Arc<OBSClient>>,
    State(pool): State<Arc<sqlx::PgPool>>,
    State(tx): State<broadcast::Sender<Event>>,
    State(reward_manager): State<Arc<RewardManager<'a, C>>>,
    Json(eventsub_body): Json<EventSubRoot>,
) -> (StatusCode, String) {
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
    user_input: &str,
    enable_dalle: bool,
    enable_stable_diffusion: bool,
) -> Result<()> {
    println!("{} {}", "Asking Chat GPT:".green(), user_input);
    let content = ask_chat_gpt(user_input, &scene.base_prompt).await?;
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
    let response = subd_openai::ask_chat_gpt(user_input, base_prompt)
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
        &command,
        reward.id,
        reward.cost,
        &event.user_name,
        &user_input,
    )
    .await?;

    // TODO: We should be reading this from some shared place
    if command == "Generate new AI CSS" {
        println!("Generating new AI CSS: {}", &user_input);

        // if the song ended, then it won't be current
        let current_song = ai_playlist::get_current_song(&pool).await;
        let id = match current_song {
            Ok(res) => res.song_id,
            Err(_) => {
                ai_playlist::find_last_played_songs(&pool, 1)
                    .await?
                    .get(1)
                    .ok_or_else(|| anyhow!("No song found"))?
                    .song_id
            }
        };

        let id = format!("{}", id);
        let _ = tokio::spawn(subd_openai::ai_styles::generate_ai_css(
            id.to_string().clone(),
            "./static/styles.css",
            user_input.clone(),
            None,
        ));
        let _ = tokio::spawn(subd_openai::ai_styles::generate_ai_js(
            id.to_string().clone(),
            "./static/styles.js",
            user_input.clone(),
            None,
        ));
        return Ok(());
    }

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
            &user_input,
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
    command: &str,
    reward_id: Uuid,
    reward_cost: i32,
    user_name: &str,
    user_input: &str,
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
        command,
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

    use uuid::Uuid;

    #[test]
    fn test_uuid_parsing() {
        let uuid_str = "ba11ad0f-dad5-c001-c001-700bac001e57";
        assert!(Uuid::parse_str(uuid_str).is_ok());
    }
}
