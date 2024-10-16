use anyhow::Result;
use models::user_roles;
use subd_types::{UserID, UserRoles};

pub mod models;
pub use models::user_roles::ModelUpdate as UserRolesUpdate;

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

    pub async fn update_roles(
        &self,
        user_id: &UserID,
        updates: UserRolesUpdate,
    ) -> Result<UserRoles> {
        // Read original model
        let model = user_roles::Model::read(&self.pool, user_id.0)
            .await?
            .unwrap_or(user_roles::Model::empty(user_id.0));

        // Send updates
        let model = model.update(&self.pool, updates).await?;

        // Return mapped to outter type
        Ok(models::user_roles::to_user_roles(model))
    }
}

#[cfg(test)]
mod tests {}
