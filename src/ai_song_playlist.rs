use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

#[database_model]
pub mod ai_song_playlist {
    use super::*;

    pub struct Model {
        pub song_id: Uuid,
        pub title: String,
        pub tags: String,
        pub prompt: String,
        pub username: String,
        pub audio_url: String,
        pub lyric: String,
        pub gpt_description_prompt: String,
        pub last_updated: Option<OffsetDateTime>,
        pub created_at: Option<OffsetDateTime>,
    }
}

impl ai_song_playlist::Model {
    #[allow(dead_code)]

    pub async fn save(&self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
                Self,
                r#"
                INSERT INTO ai_song_playlist
                (song_id, title, tags, prompt, username, audio_url, lyric, gpt_description_prompt)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING 
                    song_id, 
                    title, 
                    tags, 
                    prompt, 
                    username, 
                    audio_url, 
                    lyric, 
                    gpt_description_prompt, 
                    last_updated, 
                    created_at
                "#,
                self.song_id,
                self.title,
                self.tags,
                self.prompt,
                self.username,
                self.audio_url,
                self.lyric,
                self.gpt_description_prompt
            )
            .fetch_one(pool)
            .await?)
    }

    /// Returns the `song_id` field.
    pub fn get_song_id(&self) -> Uuid {
        self.song_id
    }

    /// Returns a reference to the `title` field.
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// Returns a reference to the `tags` field.
    pub fn get_tags(&self) -> &str {
        &self.tags
    }

    /// Returns a reference to the `prompt` field.
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }

    /// Returns a reference to the `username` field.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns a reference to the `audio_url` field.
    pub fn get_audio_url(&self) -> &str {
        &self.audio_url
    }

    /// Returns a reference to the `lyric` field.
    pub fn get_lyric(&self) -> &str {
        &self.lyric
    }

    /// Returns a reference to the `gpt_description_prompt` field.
    pub fn get_gpt_description_prompt(&self) -> &str {
        &self.gpt_description_prompt
    }
}

pub async fn find_by_id(
    pool: &sqlx::PgPool,
    song_id: Uuid,
) -> Result<ai_song_playlist::Model> {
    let res = sqlx::query!(
        "SELECT * FROM ai_song_playlist WHERE song_id = $1",
        song_id
    )
    .fetch_one(pool)
    .await?;

    let model = ai_song_playlist::Model {
        song_id,
        title: res.title,
        tags: res.tags,
        prompt: res.prompt,
        username: res.username,
        audio_url: res.audio_url,
        lyric: res.lyric,
        gpt_description_prompt: res.gpt_description_prompt,
        last_updated: res.last_updated,
        created_at: res.created_at,
    };
    Ok(model)
}
//
// pub async fn set_ai_background_theme(pool: &PgPool, theme: &str) -> Result<()> {
//     let _res = sqlx::query!(
//         "UPDATE twitch_stream_state SET ai_background_theme = $1",
//         theme,
//     )
//     .execute(pool)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn get_ai_background_theme(pool: &PgPool) -> Result<String> {
//     let res =
//         sqlx::query!("SELECT ai_background_theme FROM twitch_stream_state")
//             .fetch_one(pool)
//             .await?;
//     Ok(res.ai_background_theme.unwrap_or_default())
// }
//
// pub async fn enable_stable_diffusion(pool: &PgPool) -> Result<()> {
//     let _res = sqlx::query!(
//         "UPDATE twitch_stream_state SET enable_stable_diffusion = $1",
//         true
//     )
//     .execute(pool)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn disable_stable_diffusion(pool: &PgPool) -> Result<()> {
//     let _res = sqlx::query!(
//         "UPDATE twitch_stream_state SET enable_stable_diffusion = $1",
//         false
//     )
//     .execute(pool)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn turn_off_dalle_mode(pool: &PgPool) -> Result<()> {
//     let _res =
//         sqlx::query!("UPDATE twitch_stream_state SET dalle_mode = $1", false)
//             .execute(pool)
//             .await?;
//
//     Ok(())
// }
//
// pub async fn turn_on_dalle_mode(pool: &PgPool) -> Result<()> {
//     let _res =
//         sqlx::query!("UPDATE twitch_stream_state SET dalle_mode = $1", true)
//             .execute(pool)
//             .await?;
//
//     Ok(())
// }
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
// pub async fn turn_on_global_voice(pool: &PgPool) -> Result<()> {
//     let _res =
//         sqlx::query!("UPDATE twitch_stream_state SET global_voice = $1", true)
//             .execute(pool)
//             .await?;
//
//     Ok(())
// }
//
// pub async fn update_implicit_soundeffects(pool: &PgPool) -> Result<()> {
//     let state = get_twitch_state(pool).await?;
//     let soundeffects = !state.implicit_soundeffects;
//     let _res = sqlx::query!(
//         "UPDATE twitch_stream_state SET implicit_soundeffects = $1",
//         soundeffects
//     )
//     .execute(pool)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn update_explicit_soundeffects(pool: &PgPool) -> Result<()> {
//     let state = get_twitch_state(pool).await?;
//     let soundeffects = !state.explicit_soundeffects;
//
//     let _res = sqlx::query!(
//         "UPDATE twitch_stream_state SET explicit_soundeffects = $1",
//         soundeffects
//     )
//     .execute(pool)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn get_twitch_state(
//     pool: &PgPool,
// ) -> Result<twitch_stream_state::Model> {
//     let res = sqlx::query!("SELECT * FROM twitch_stream_state")
//         .fetch_one(pool)
//         .await?;
//     let model = twitch_stream_state::Model {
//         sub_only_tts: res.sub_only_tts,
//         explicit_soundeffects: res.explicit_soundeffects,
//         implicit_soundeffects: res.implicit_soundeffects,
//         global_voice: res.global_voice,
//         dalle_mode: res.dalle_mode,
//         dalle_model: res.dalle_model,
//         enable_stable_diffusion: res.enable_stable_diffusion,
//     };
//     Ok(model)
// }
