use anyhow::Result;
use subd_types::{TwitchUserID, UserID, UserRoles};

#[allow(dead_code)]
pub struct TwitchUser {
    id: TwitchUserID,
    user_id: UserID,
    login: String,
    name: String,
}

pub struct Service {
    pool: sqlx::PgPool,
    users: user_service::Service,
}

impl Service {
    pub async fn new(pool: sqlx::PgPool, users: user_service::Service) -> Self {
        Self { pool, users }
    }

    pub async fn create(&self) -> TwitchUser {
        todo!()
    }

    pub async fn get(&self, _id: TwitchUserID) -> Result<Option<TwitchUser>> {
        println!("{:?}", self.pool);
        todo!()
    }

    pub async fn get_user_id(
        &self,
        id: TwitchUserID,
    ) -> Result<Option<UserID>> {
        Ok(sqlx::query!(
            "SELECT user_id FROM twitch_users WHERE twitch_user_id = $1",
            id.0
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|x| UserID(x.user_id)))
    }

    pub async fn update_user_roles(
        &self,
        user_id: &UserID,
        user_roles: &UserRoles,
    ) -> Result<UserRoles> {
        self.users
            .update_roles(
                user_id,
                user_service::UserRolesUpdate {
                    is_twitch_mod: Some(user_roles.is_twitch_mod()),
                    is_twitch_vip: Some(user_roles.is_twitch_vip()),
                    is_twitch_founder: Some(user_roles.is_twitch_founder()),
                    is_twitch_sub: Some(user_roles.is_twitch_sub()),
                    is_twitch_staff: Some(user_roles.is_twitch_staff()),
                    ..Default::default()
                },
            )
            .await
    }

    // pub fn get_user_roles_from_msg(&self,
}

#[cfg(test)]
mod tests {}
