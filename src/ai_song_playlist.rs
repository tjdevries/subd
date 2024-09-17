use crate::ai_songs::ai_songs;
use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

#[database_model]
pub mod ai_song_playlist {
    use super::*;

    pub struct Model {
        pub playlist_id: Uuid,
        pub song_id: Uuid,
        pub created_at: Option<OffsetDateTime>,
        pub played_at: Option<OffsetDateTime>,
    }
}

impl ai_song_playlist::Model {
    #[allow(dead_code)]
    pub async fn save(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO ai_song_playlist
            (playlist_id, song_id, created_at, played_at)
            VALUES ($1, $2, $3, $4)
            RETURNING
                playlist_id,
                song_id,
                created_at,
                played_at
            "#,
            self.playlist_id,
            self.song_id,
            self.created_at,
            self.played_at
        )
        .fetch_one(pool)
        .await?)
    }
}
