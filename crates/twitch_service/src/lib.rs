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
    db: sqlx::PgPool,
}

impl Service {
    pub async fn new(db: sqlx::PgPool) -> Self {
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
