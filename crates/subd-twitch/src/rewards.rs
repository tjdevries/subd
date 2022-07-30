use anyhow::Result;
use twitch_api2::{
    helix::points::{
        CreateCustomRewardBody, CreateCustomRewardRequest, UpdateCustomRewardBody,
        UpdateCustomRewardRequest,
    },
    twitch_oauth2::UserToken,
    HelixClient,
};

pub struct RewardManager<'a, C>
where
    C: twitch_api2::HttpClient<'a>,
{
    client: &'a HelixClient<'a, C>,
    token: &'a UserToken,
}

impl<'a, C> RewardManager<'a, C>
where
    C: twitch_api2::HttpClient<'a>,
{
    pub fn new(client: &'a HelixClient<'a, C>, token: &'a UserToken) -> Self
    where
        C: twitch_api2::HttpClient<'a>,
    {
        Self { client, token }
    }

    pub async fn set_reward_status(&self, id: &str, status: bool) -> Result<()> {
        let req = UpdateCustomRewardRequest::builder()
            .broadcaster_id(self.token.user_id.clone())
            .id(id)
            .build();

        let body = UpdateCustomRewardBody::builder()
            .is_enabled(Some(status))
            .build();

        self.client.req_patch(req, body, self.token).await?;

        Ok(())
    }

    pub async fn create_reward(&self, title: &str, cost: usize) -> Result<()> {
        let req = CreateCustomRewardRequest::builder()
            .broadcaster_id(self.token.user_id.clone())
            .build();

        let body = CreateCustomRewardBody::builder()
            .title(title)
            .cost(cost)
            .build();

        self.client.req_post(req, body, self.token).await?;

        Ok(())
    }
}
