// use anyhow::Error;
use anyhow::Result;
use sqlx::postgres::PgQueryResult;
use sqlx::postgres::PgRow;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::PgPool;
use subd_macros::database_model;

#[database_model]
pub mod redemptions {
    use super::*;

    pub struct Model {
        pub title: String,
        pub cost: i32,
        pub user_name: String,
        pub reward_id: Uuid,
        pub twitch_id: Uuid,

        pub created_at: Option<OffsetDateTime>,

        // This might need to be text
        // optional might FUCKING US
        pub user_input: Option<String>,
    }
}

impl redemptions::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO redemptions 
            (title, cost, user_name, twitch_id, reward_id, user_input, created_at)
            VALUES ( $1, $2, $3, $4, $5, $6, $7)
            RETURNING title, cost, user_name, twitch_id, reward_id, user_input, created_at
        "#,
            self.title,
            self.cost,
            self.user_name,
            self.twitch_id,
            self.reward_id,
            self.user_input,
            self.created_at,
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn save_redemptions(
    pool: &sqlx::PgPool,
    title: &str,
    cost: i32,
    user_name: &str,
    twitch_id: Uuid,
    reward_id: Uuid,
    user_input: &str,
) -> Result<PgQueryResult> {
    // Watch out, we have confused up the idea of reward_id and twitch_id
    // we have unique constraint on reward_id, which is the static ID to a reward in Twitch
    // it's not pure request
    sqlx::query!(
        r#"INSERT INTO redemptions (title, cost, user_name, twitch_id, reward_id, user_input)
       VALUES ( $1, $2, $3, $4, $5, $6 )"#,
        title,
        cost,
        user_name,
        reward_id,
        twitch_id,
        user_input,
    )
    .execute(pool)
    .await.map_err(|e| e.into())
}

pub async fn find_redemption_by_twitch_id(
    pool: &PgPool,
    twitch_id: Uuid,
) -> Result<PgRow, sqlx::Error> {
    sqlx::query("SELECT * FROM redemptions WHERE twitch_id = $1")
        .bind(twitch_id)
        .fetch_one(pool)
        .await
}

pub async fn find_redemption_by_reward_id(
    pool: &PgPool,
    reward_id: Uuid,
) -> Result<PgRow, sqlx::Error> {
    sqlx::query("SELECT * FROM redemptions WHERE reward_id = $1")
        .bind(reward_id)
        .fetch_one(pool)
        .await
}

pub async fn find_recent_rewards(pool: &PgPool) -> Result<Vec<Uuid>> {
    // I want all from the Last hour
    let res = sqlx::query!("SELECT reward_id FROM redemptions WHERE created_at >= now() - interval '60 minutes'")
        .fetch_all(pool)
        .await?;

    let ids: Vec<Uuid> = res.iter().map(|r| r.reward_id).collect();
    Ok(ids)
}

//
// pub async fn turn_off_global_voice(pool: &PgPool) -> Result<()> {
//     let _res =
//         sqlx::query!("UPDATE twitch_stream_state SET global_voice = $1", false)
//             .execute(pool)
//             .await?;
//
//     Ok(())
// }
//
