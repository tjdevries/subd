use anyhow::Result;
// use sqlx::types::Uuid;

use twitch_api::helix::points::{
    CreateCustomRewardBody, CreateCustomRewardRequest, UpdateCustomRewardBody,
    UpdateCustomRewardRequest,
};
use twitch_api::helix::HelixClient;
use twitch_api::helix::{
    points::create_custom_rewards, points::update_custom_reward,
};
use twitch_api::twitch_oauth2::UserToken;

pub struct RewardManager2<'a, C>
where
    C: twitch_api::HttpClient,
{
    client: HelixClient<'a, C>,
    broadcaster_id: String,
    token: UserToken,
}

impl<'a, C> RewardManager2<'a, C>
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

pub struct RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    client: &'a HelixClient<'a, C>,
    broadcaster_id: &'a str,
    token: &'a UserToken,
}

impl<'a, C> RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    pub fn new(
        client: &'a HelixClient<'a, C>,
        token: &'a UserToken,
        broadcaster_id: &'a str,
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

    pub async fn set_reward_status(
        &self,
        _id: &str,
        _status: bool,
    ) -> Result<()> {
        Ok(())
    }

    // pub async fn update_all(&self, cost: usize) -> Result<()> {
    //     // I have to iterate through all
    //     let reward_id = id.clone();
    //     let request = update_custom_reward::UpdateCustomRewardRequest::new(self.broadcaster_id, reward_id);
    //     let mut body = update_custom_reward::UpdateCustomRewardBody::default();
    //     body.cost = Some(cost);
    //     let response: update_custom_reward::UpdateCustomReward = self.client.req_patch(request, body, self.token).await?.data;
    //     Ok(())
    // }

    pub async fn update_reward(&self, id: String, cost: usize) -> Result<()> {
        let reward_id = id.clone();
        let request =
            UpdateCustomRewardRequest::new(self.broadcaster_id, reward_id);
        let mut body = UpdateCustomRewardBody::default();
        body.cost = Some(cost);
        let _response: update_custom_reward::UpdateCustomReward =
            self.client.req_patch(request, body, self.token).await?.data;
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
            CreateCustomRewardRequest::broadcaster_id(self.broadcaster_id);
        let response: create_custom_rewards::CreateCustomRewardResponse =
            self.client.req_post(request, body, self.token).await?.data;

        // I
        let id = format!("{}", response.id);

        Ok(id)

        // response.id

        // I need to save in our DB!

        // println!("Response: {:?}", response);
        // let req = CreateCustomRewardRequest::builder()
        //     .broadcaster_id(self.token.user_id.clone())
        //     .build();
        //
        // let body = CreateCustomRewardBody::builder()
        //     .title(title)
        //     .cost(cost)
        //     .build();
        //
        // self.client.req_post(req, body, self.token).await?;
    }
}
