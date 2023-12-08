// use std::collections::VecDeque;
// use anyhow::Error;
// use sqlx::postgres::PgRow;
use anyhow::Result;
use sqlx::types::Uuid;
use sqlx::PgPool;
use subd_macros::database_model;

#[database_model]
pub mod twitch_rewards {
    use super::*;

    pub struct Model {
        pub title: String,
        pub cost: i32,
        pub twitch_id: Uuid,
        pub enabled: bool,
    }
}

impl twitch_rewards::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO twitch_rewards
            (title, cost, twitch_id, enabled)
            VALUES ( $1, $2, $3, $4)
            RETURNING title, cost, twitch_id, enabled
        "#,
            self.title,
            self.cost,
            self.twitch_id,
            self.enabled,
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn save_twitch_rewards(
    pool: &sqlx::PgPool,
    title: String,
    cost: usize,
    twitch_id: Uuid,
    enabled: bool,
) -> Result<()> {
    let icost = cost as i32;
    sqlx::query!(
        r#"
        INSERT INTO twitch_rewards (title, cost, twitch_id, enabled)
        VALUES ( $1, $2, $3, $4)
       "#,
        title,
        icost,
        twitch_id,
        enabled,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_cost_of_all(
    pool: &PgPool,
    cost: i32,
) -> Result<Vec<String>> {
    let mut ids: Vec<String> = vec![];
    let res = sqlx::query!("SELECT twitch_id FROM twitch_rewards")
        .fetch_all(pool)
        .await?;

    for item in res {
        let _res = sqlx::query!(
            "UPDATE twitch_rewards SET cost = $1 WHERE twitch_id = $2",
            cost,
            item.twitch_id,
        )
        .execute(pool)
        .await?;
        ids.push(item.twitch_id.to_string());
    }

    // We need to call the other thang

    Ok(ids)
}

pub async fn find_by_title(
    pool: &PgPool,
    title: String,
) -> Result<twitch_rewards::Model> {
    let res =
        sqlx::query!("SELECT * FROM twitch_rewards WHERE title = $1", title)
            .fetch_one(pool)
            .await?;

    let model = twitch_rewards::Model {
        title,
        cost: res.cost,
        twitch_id: res.twitch_id,
        enabled: res.enabled,
    };
    return Ok(model);
}

pub async fn find_all_ids_except(
    pool: &sqlx::PgPool,
    current_reward_id: Uuid,
) -> Result<Vec<(Uuid, i32)>> {
    let res = sqlx::query!(
        r#"
            SELECT tw.twitch_id, tw.cost
            FROM twitch_rewards tw
            WHERE NOT EXISTS (
                SELECT 1
                FROM redemptions red
                WHERE red.reward_id = tw.twitch_id 
                AND red.reward_id != $1
                AND red.created_at >= now() - interval '60 minutes'
            );
        "#,
        current_reward_id,
    )
    .fetch_all(pool)
    .await?;

    let uuids = res.iter().map(|r| (r.twitch_id, r.cost)).collect();
    Ok(uuids)
}

pub async fn find_all_ids_for_twitch_id(
    pool: &sqlx::PgPool,
    twitch_id: Uuid,
) -> Result<Vec<(Uuid, i32)>> {
    let id = twitch_id.to_string();
    let res = sqlx::query!( "SELECT twitch_id, cost FROM twitch_rewards WHERE twitch_id::text != $1", id).fetch_all(pool).await?;

    let uuids = res.iter().map(|r| (r.twitch_id, r.cost)).collect();
    return Ok(uuids);
}

pub async fn find_by_id(
    pool: &sqlx::PgPool,
    twitch_id: Uuid,
) -> Result<twitch_rewards::Model> {
    let res = sqlx::query!(
        "SELECT * FROM twitch_rewards WHERE twitch_id = $1",
        twitch_id
    )
    .fetch_one(pool)
    .await?;

    let model = twitch_rewards::Model {
        title: res.title,
        cost: res.cost,
        twitch_id: res.twitch_id,
        enabled: res.enabled,
    };
    Ok(model)
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

pub async fn update_cost_by_id(
    pool: &PgPool,
    id: Uuid,
    cost: i32,
) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_rewards SET cost = $1 WHERE twitch_id = $2",
        cost,
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_cost(
    pool: &PgPool,
    title: String,
    cost: i32,
) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_rewards SET cost = $1 WHERE title = $2",
        cost,
        title,
    )
    .execute(pool)
    .await?;

    Ok(())
}
