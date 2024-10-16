use anyhow::Result;
use std::env;
use twitch_api::helix::points::{
    CreateCustomRewardBody, CreateCustomRewardRequest, UpdateCustomRewardBody,
    UpdateCustomRewardRequest,
};
use twitch_api::helix::HelixClient;
use twitch_api::helix::{
    points::create_custom_rewards, points::update_custom_reward,
};
use twitch_api::twitch_oauth2::UserToken;

pub async fn build_reward_manager<'a>(
) -> Result<RewardManager<'a, reqwest::Client>> {
    let twitch_user_access_token =
        env::var("TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN")
            .expect("missing env var TWITCH_CHANNEL_REWARD_USER_ACCESS_TOKEN");
    let broadcaster_id: String = env::var("TWITCH_BROADCAST_ID")
        .expect("missing env var TWITCH_BROADCAST_ID");

    let reqwest = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let twitch_reward_client: HelixClient<reqwest::Client> = HelixClient::new();
    let token = UserToken::from_existing(
        &reqwest,
        twitch_user_access_token.into(),
        None,
        None,
    )
    .await?;

    let reward_manager =
        RewardManager::new(twitch_reward_client, token, broadcaster_id);
    Ok(reward_manager)
}

pub struct RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    client: HelixClient<'a, C>,
    broadcaster_id: String,
    token: UserToken,
}

impl<'a, C> RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    pub fn new(
        client: HelixClient<'a, C>,
        token: UserToken,
        broadcaster_id: String,
    ) -> Self
    where
        C: twitch_api::HttpClient,
    {
        Self {
            client,
            broadcaster_id,
            token,
        }
    }

    pub async fn update_reward(&self, id: String, cost: usize) -> Result<()> {
        let reward_id = id.clone();
        let request =
            UpdateCustomRewardRequest::new(&self.broadcaster_id, reward_id);
        let mut body = UpdateCustomRewardBody::default();
        body.cost = Some(cost);
        let _response: update_custom_reward::UpdateCustomReward = self
            .client
            .req_patch(request, body, &self.token)
            .await?
            .data;
        Ok(())
    }

    pub async fn create_reward(
        &self,
        title: &str,
        cost: usize,
    ) -> Result<String> {
        let mut body = CreateCustomRewardBody::new(title, cost);

        body.is_user_input_required = Some(true);

        let request =
            CreateCustomRewardRequest::broadcaster_id(&self.broadcaster_id);
        let response: create_custom_rewards::CreateCustomRewardResponse =
            self.client.req_post(request, body, &self.token).await?.data;
        let id = format!("{}", response.id);
        Ok(id)
    }
}
