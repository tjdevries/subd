use anyhow::Result;
use sqlx::PgPool;
use sqlx::types::Uuid;
use subd_macros::database_model;

#[database_model]
pub mod redemptions {
    use super::*;

    pub struct Model {
        pub title: String,
        pub cost: i32,
        pub user_name: String,
        pub reward_id: Uuid,
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
            (title, cost, user_name, reward_id, user_input)
            VALUES ( $1, $2, $3, $4, $5)
            RETURNING title, cost, user_name, reward_id, user_input
        "#,
            self.title,
            self.cost,
            self.user_name,
            self.reward_id,
            self.user_input
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn save_redemptions(
    pool: &sqlx::PgPool,
    title: String,
    cost: i32,
    user_name: String,
    reward_id: Uuid, 
    user_input: String,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO redemptions (title, cost, user_name, reward_id, user_input)
       VALUES ( $1, $2, $3, $4, $5 )"#,
        title,
        cost,
        user_name,
        reward_id,
        user_input,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_redemption(
    pool: &PgPool,
    reward_id: Uuid, 
) -> Result<redemptions::Model> {
    // sqlx::query!("DELETE FROM user_theme_songs WHERE user_id = $1", user_id)
    let res = sqlx::query!("SELECT * FROM redemptions WHERE reward_id = $1", reward_id)
        .fetch_one(pool)
        .await?;

    // Res return
    let model = redemptions::Model {
        title: todo!(),
        cost: todo!(),
        user_name: todo!(),
        reward_id: todo!(),
        user_input: todo!(),
        
        // sub_only_tts: res.sub_only_tts,
        // explicit_soundeffects: res.explicit_soundeffects,
        // implicit_soundeffects: res.implicit_soundeffects,
        // global_voice: res.global_voice,
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
//
