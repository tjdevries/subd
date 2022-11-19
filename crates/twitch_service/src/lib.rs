use anyhow::Result;
use subd_types::{TwitchUserID, UserID};
use user_service::User;

#[allow(dead_code)]
pub struct TwitchUser {
    id: TwitchUserID,
    user_id: UserID,
    login: String,
    name: String,
}

pub struct Service {
    db: sqlx::SqlitePool,
}

impl Service {
    pub async fn new(db: sqlx::SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create(&self) -> TwitchUser {
        todo!()
    }

    pub async fn get(&self, _id: TwitchUserID) -> Result<Option<TwitchUser>> {
        println!("{:?}", self.db);
        todo!()
    }
}

#[cfg(test)]
mod tests {}
