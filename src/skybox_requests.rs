use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;

#[database_model]
pub mod skybox_request {
    use super::*;

    pub struct Model {
        pub blockade_id: i32,
        pub prompt: String,
        pub skybox_style_id: i32,
        pub file_url: Option<String>,

        // TODO: Do I want to depend on Postgresql FKs for users?
        pub username: String,
        pub created_at: Option<OffsetDateTime>,
        pub completed_at: Option<OffsetDateTime>,
    }
}

impl skybox_request::Model {
    #[allow(dead_code)]
    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO skybox_requests (blockade_id, prompt, skybox_style_id, file_url, username, created_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING blockade_id, prompt, skybox_style_id, file_url, username, created_at, completed_at
            "#,
            self.blockade_id,
            self.prompt,
            self.skybox_style_id,
            self.file_url,
            self.username,
            self.created_at,
            self.completed_at
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn save_skybox_request(
    pool: &sqlx::PgPool,
    blockade_id: i32,
    prompt: String,
    skybox_style_id: i32,
    username: String,
) -> Result<()> {
    sqlx::query!(
    r#"
        INSERT INTO skybox_requests (blockade_id, prompt, skybox_style_id, username)
           VALUES ( $1, $2, $3, $4 )
    "#, blockade_id, prompt, skybox_style_id, username,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn uncompleted_skybox_requests(
    pool: &sqlx::PgPool,
) -> Result<Vec<skybox_request::Model>> {
    let uncompleted: Vec<skybox_request::Model> = sqlx::query_as!(
        skybox_request::Model,
        "SELECT * FROM skybox_requests WHERE completed_at IS NULL"
    )
    .fetch_all(pool)
    .await?;
    Ok(uncompleted)
}

pub async fn update_skybox_request(
    pool: &sqlx::PgPool,
    blockade_id: i32,
    file_url: String,
    completed_at: OffsetDateTime,
) -> Result<()> {
    let res = sqlx::query!(
            "UPDATE skybox_requests SET file_url = $1, completed_at = $2 WHERE blockade_id = $3",
            file_url,
            completed_at,
            blockade_id,
        ).execute(pool).await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::skybox;
    use crate::skybox_requests;
    use subd_db::get_db_pool;

    #[tokio::test]
    async fn test_model() {
        // TODO: Move this somewhere shared
        let pool = get_db_pool().await;

        let blockade_id = 9612607;
        let prompt = "Cool STuff".to_string();
        let skybox_style_id = 1;
        let username = "Beginbot".to_string();
        let _ = skybox_requests::save_skybox_request(
            &pool,
            blockade_id,
            prompt,
            skybox_style_id,
            username,
        )
        .await;

        let uncompleted = skybox_requests::uncompleted_skybox_requests(&pool)
            .await
            .unwrap();
        assert_eq!(uncompleted.len(), 1);

        match skybox::check_skybox_status(blockade_id).await {
            Ok(skybox_status) => {
                let file_url = skybox_status.file_url;
                let completed_at = sqlx::types::time::OffsetDateTime::now_utc();

                let _ = skybox_requests::update_skybox_request(
                    &pool,
                    blockade_id,
                    file_url,
                    completed_at,
                )
                .await;
                let uncompleted =
                    skybox_requests::uncompleted_skybox_requests(&pool)
                        .await
                        .unwrap();
                assert_eq!(uncompleted.len(), 0);
            }
            Err(_e) => {
                assert!(false);
            }
        };
    }
}
