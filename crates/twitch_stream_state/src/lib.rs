// Code will go here
use anyhow::Result;
use sqlx::types::Uuid;
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
        pub enable_stable_diffusion: bool,
    }
}

impl twitch_stream_state::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO twitch_stream_state
            (sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice, dalle_mode, dalle_model, enable_stable_diffusion)
            VALUES ( $1, $2, $3, $4, $5, $6, $7)
            RETURNING sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice, dalle_mode, dalle_model, enable_stable_diffusion
        "#,
            true,
            true,
            true,
            false,
            true,
            "dalle-3".to_string(),
            true,
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn set_ai_background_theme(pool: &PgPool, theme: &str) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET ai_background_theme = $1",
        theme,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Should I keep this a UUID
pub async fn get_current_song_id(pool: &PgPool) -> Result<String> {
    let res = sqlx::query!("SELECT current_song_id FROM twitch_stream_state")
        .fetch_one(pool)
        .await?;
    Ok(res.current_song_id.unwrap_or_default().to_string())
}

pub async fn get_ai_background_theme(pool: &PgPool) -> Result<String> {
    let res =
        sqlx::query!("SELECT ai_background_theme FROM twitch_stream_state")
            .fetch_one(pool)
            .await?;
    Ok(res.ai_background_theme.unwrap_or_default())
}

pub async fn enable_stable_diffusion(pool: &PgPool) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET enable_stable_diffusion = $1",
        true
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn disable_stable_diffusion(pool: &PgPool) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET enable_stable_diffusion = $1",
        false
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn turn_off_dalle_mode(pool: &PgPool) -> Result<()> {
    let _res =
        sqlx::query!("UPDATE twitch_stream_state SET dalle_mode = $1", false)
            .execute(pool)
            .await?;

    Ok(())
}

// pub async fn find_current_song(
//     pool: &PgPool,
// ) -> Result<PgRow, sqlx::Error> {
//     sqlx::query("SELECT * FROM twitch_stream_state")
//         .fetch_one(pool)
//         .await
// }

pub async fn update_current_song(
    pool: &PgPool,
    current_song_id: Uuid,
) -> Result<()> {
    let _res = sqlx::query!(
        "UPDATE twitch_stream_state SET current_song_id = $1",
        current_song_id
    )
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
        .fetch_optional(pool)
        .await?;

    let model = if let Some(res) = res {
        twitch_stream_state::Model {
            sub_only_tts: res.sub_only_tts,
            explicit_soundeffects: res.explicit_soundeffects,
            implicit_soundeffects: res.implicit_soundeffects,
            global_voice: res.global_voice,
            dalle_mode: res.dalle_mode,
            dalle_model: res.dalle_model,
            enable_stable_diffusion: res.enable_stable_diffusion,
        }
    } else {
        twitch_stream_state::Model {
            sub_only_tts: true,
            explicit_soundeffects: true,
            implicit_soundeffects: true,
            global_voice: false,
            dalle_mode: true,
            dalle_model: "dalle-3".to_string(),
            enable_stable_diffusion: true,
        }
    };
    Ok(model)
}
