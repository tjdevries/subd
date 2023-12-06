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
        pub global_voice: bool,
        pub dalle_mode: bool,
        pub dalle_model: String,
    }
}

impl twitch_stream_state::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO twitch_stream_state
            (sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice, dalle_mode, dalle_model)
            VALUES ( $1, $2, $3, $4, $5, $6)
            RETURNING sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice, dalle_mode, dalle_model
        "#,
            true,
            true,
            true,
            false,
            true,
            "gpt-3.5-turbo".to_string(),
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn turn_off_dalle_mode(pool: &PgPool) -> Result<()> {
    let _res =
        sqlx::query!("UPDATE twitch_stream_state SET dalle_mode = $1", false)
            .execute(pool)
            .await?;

    Ok(())
}

pub async fn turn_on_dalle_mode(pool: &PgPool) -> Result<()> {
    let _res =
        sqlx::query!("UPDATE twitch_stream_state SET dalle_mode = $1", true)
            .execute(pool)
            .await?;

    Ok(())
}

pub async fn turn_off_global_voice(pool: &PgPool) -> Result<()> {
    let _res =
        sqlx::query!("UPDATE twitch_stream_state SET global_voice = $1", false)
            .execute(pool)
            .await?;

    Ok(())
}

pub async fn turn_on_global_voice(pool: &PgPool) -> Result<()> {
    let _res =
        sqlx::query!("UPDATE twitch_stream_state SET global_voice = $1", true)
            .execute(pool)
            .await?;

    Ok(())
}

pub async fn update_implicit_soundeffects(pool: &PgPool) -> Result<()> {
    let state = get_twitch_state(pool).await?;
    let soundeffects = !state.implicit_soundeffects;
    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET implicit_soundeffects = $1",
        soundeffects
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_explicit_soundeffects(pool: &PgPool) -> Result<()> {
    let state = get_twitch_state(pool).await?;
    let soundeffects = !state.explicit_soundeffects;

    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET explicit_soundeffects = $1",
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
        global_voice: res.global_voice,
        dalle_mode: res.dalle_mode,
        dalle_model: res.dalle_model,
    };
    Ok(model)
}
