use anyhow::Result;
use subd_types::{TwitchUserID, UserID};

#[allow(dead_code)]
pub struct TwitchUser {
    id: TwitchUserID,
    user_id: UserID,
    login: String,
    name: String,
}

pub struct Service {
    db: sqlx::PgConnection,
}

impl Service {
    pub async fn new(db: sqlx::PgConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self) -> TwitchUser {
        todo!()
    }

    pub async fn get(&self, _id: TwitchUserID) -> Result<Option<TwitchUser>> {
        println!("{:?}", self.db);
        todo!()
    }

    pub async fn get_user_id(
        &mut self,
        id: TwitchUserID,
    ) -> Result<Option<UserID>> {
        Ok(sqlx::query!(
            "SELECT user_id FROM twitch_users WHERE twitch_user_id = $1",
            id.0
        )
        .fetch_optional(&mut self.db)
        .await?
        .map(|x| UserID(x.user_id)))
    }
}

#[cfg(test)]
mod tests {}
