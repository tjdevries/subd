use anyhow::Result;
use sqlx::PgPool;
use subd_macros::database_model;

#[database_model]
pub mod twitch_stream_state {
    use super::*;

    pub struct Model {
        pub sub_only_tts: bool,
        pub explicit_soundeffects: bool,
        pub implicit_soundeffects: bool,
    }
}

// TODO: Take in Random
impl twitch_stream_state::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO twitch_stream_state
            (sub_only_tts, explicit_soundeffects, implicit_soundeffects)
            VALUES ( $1, $2, $3 )
            RETURNING sub_only_tts, explicit_soundeffects, implicit_soundeffects
        "#,
            true,
            true,
            true,
        )
        .fetch_one(pool)
        .await?)
    }
}
pub async fn update_implicit_soundeffects(
    soundeffects: bool,
    pool: &PgPool,
) -> Result<()> {
    let res = sqlx::query!(
        "UPDATE twitch_stream_state SET implicit_soundeffects = $1",
        soundeffects
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_twitch_state(
    pool: &PgPool,
) -> Result<twitch_stream_state::Model> {
    let res = sqlx::query!("SELECT * FROM twitch_stream_state")
        .fetch_one(pool)
        .await?;
    let model = twitch_stream_state::Model {
        sub_only_tts: res.sub_only_tts,
        explicit_soundeffects: res.explicit_soundeffects,
        implicit_soundeffects: res.implicit_soundeffects,
    };
    Ok(model)
}
