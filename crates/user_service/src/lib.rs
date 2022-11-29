use anyhow::Result;
use subd_types::{UserID, UserRoles};

mod models;

pub struct User {
    pub id: UserID,
}

pub struct Service {
    pool: sqlx::PgPool,
}

impl Service {
    pub async fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self) -> Result<User> {
        todo!()
    }

    pub async fn get(&self, _id: UserID) -> Result<Option<User>> {
        println!("{:?}", self.pool);
        todo!()
    }

    pub async fn get_roles(
        &mut self,
        id: &UserID,
    ) -> Result<Option<UserRoles>> {
        Ok(models::user_roles::Model::read(&self.pool, id.0)
            .await?
            .map(models::user_roles::to_user_roles))
    }
}

#[cfg(test)]
mod tests {}
