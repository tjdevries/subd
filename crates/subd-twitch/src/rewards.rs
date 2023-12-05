use anyhow::Result;

use twitch_api::helix::HelixClient;
use twitch_api::{
    helix::points::{
        CreateCustomRewardBody, CreateCustomRewardRequest,
        UpdateCustomRewardBody, UpdateCustomRewardRequest,
    },
    twitch_oauth2::UserToken,
};

pub struct RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    client: &'a HelixClient<'a, C>,
    token: &'a UserToken,
}

impl<'a, C> RewardManager<'a, C>
where
    C: twitch_api::HttpClient,
{
    pub fn new(client: &'a HelixClient<'a, C>, token: &'a UserToken) -> Self
    where
        C: twitch_api::HttpClient,
    {
        Self { client, token }
    }

    pub async fn set_reward_status(
        &self,
        id: &str,
        status: bool,
    ) -> Result<()> {
        // let client: helix::HelixClient<'static, client::DummyHttpClient> = helix::HelixClient::default();
        // let token = twitch_oauth2::AccessToken::new("validtoken".to_string());
        // let token = twitch_oauth2::UserToken::from_existing(&client, token, None, None).await?;
        // let request = update_custom_reward::UpdateCustomRewardRequest::new("274637212", "reward-id");
        // let mut body = update_custom_reward::UpdateCustomRewardBody::default();
        // body.cost = Some(501);
        // body.title = Some("hydrate but differently now!".into());
        // let response: update_custom_reward::UpdateCustomReward = self.client.req_patch(request, body, &token).await?.data;

        //
        // let req = UpdateCustomRewardRequest::builder()
        //     .broadcaster_id(self.token.user_id.clone())
        //     .id(id)
        //     .build();
        //
        // let body = UpdateCustomRewardBody::builder()
        //     .is_enabled(Some(status))
        //     .build();
        //
        // self.client.req_patch(req, body, self.token).await?;

        Ok(())
    }

    pub async fn create_reward(&self, title: &str, cost: usize) -> Result<()> {
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
