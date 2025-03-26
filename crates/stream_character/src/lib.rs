use anyhow::Result;
use sqlx::PgPool;
use subd_macros::database_model;

#[database_model]
pub mod user_stream_character_information {
    use super::*;

    pub struct Model {
        pub username: String,
        pub obs_character: String,
        pub voice: String,
        pub random: bool,
    }
}

impl user_stream_character_information::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO user_stream_character_information
            (username, obs_character, voice)
            VALUES ( $1, $2, $3 )
            ON CONFLICT (username)
            DO UPDATE SET
            obs_character = $2,
            voice = $3
            RETURNING username, obs_character, voice, random
        "#,
            self.username,
            self.obs_character,
            self.voice
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn get_voice_from_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<String>> {
    let res = sqlx::query!(
        "SELECT voice FROM user_stream_character_information WHERE username = $1",
        username
    ).fetch_optional(pool).await?;
    Ok(res.map(|r| r.voice))
}
