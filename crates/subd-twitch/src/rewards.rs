use anyhow::Result;

use twitch_api::helix::points::{
    CreateCustomRewardBody, CreateCustomRewardRequest, UpdateCustomRewardBody,
    UpdateCustomRewardRequest,
};
use twitch_api::helix::HelixClient;
use twitch_api::helix::{
    self, points::create_custom_rewards, points::update_custom_reward,
};
use twitch_api::twitch_oauth2::UserToken;

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

    pub async fn create_reward(&self, title: &str, cost: usize) -> Result<()> {
        let mut body =
            create_custom_rewards::CreateCustomRewardBody::new(title, cost);
        let request =
            create_custom_rewards::CreateCustomRewardRequest::broadcaster_id(
                self.broadcaster_id,
            );
        let response: create_custom_rewards::CreateCustomRewardResponse =
            self.client.req_post(request, body, self.token).await?.data;
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

        Ok(())
    }
}
