use anyhow::Result;
use subd_types::{UserID, UserRoles};

mod models;

pub struct User {
    pub id: UserID,
}

pub struct Service {
    conn: sqlx::PgConnection,
}

impl Service {
    pub async fn new(conn: sqlx::PgConnection) -> Self {
        Self { conn }
    }

    pub async fn create(&self) -> Result<User> {
        todo!()
    }

    pub async fn get(&self, _id: UserID) -> Result<Option<User>> {
        println!("{:?}", self.conn);
        todo!()
    }

    pub async fn get_roles(
        &mut self,
        id: &UserID,
    ) -> Result<Option<UserRoles>> {
        Ok(models::user_roles::Model::read(&mut self.conn, id)
            .await?
            .map(models::user_roles::to_user_roles))
    }
}

#[cfg(test)]
mod tests {}
