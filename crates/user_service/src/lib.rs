use anyhow::Result;
use subd_types::UserID;

pub struct User {
    pub id: UserID,
}

pub struct Service {
    db: sqlx::PgPool,
}

impl Service {
    pub async fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self) -> Result<User> {
        todo!()
    }

    pub async fn get(&self, _id: UserID) -> Result<Option<User>> {
        println!("{:?}", self.db);
        todo!()
    }
}

#[cfg(test)]
mod tests {}
