use anyhow::Result;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

#[database_model]
pub mod ai_song_vote {
    use super::*;

    pub struct Model {
        pub song_id: Uuid,
        pub user_id: Uuid,
        pub good_song: bool,
        pub score: Option<BigDecimal>,
    }
}

impl ai_song_vote::Model {
    #[allow(dead_code)]
    pub async fn save(&self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
                INSERT INTO ai_song_vote
                (song_id, user_id, good_song, score)
                VALUES ($1, $2, $3, $4)
                RETURNING
                    song_id,
                    user_id,
                    good_song,
                    score
                "#,
            self.song_id,
            self.user_id,
            self.good_song,
            self.score,
        )
        .fetch_one(pool)
        .await?)
    }

    /// Returns the `song_id` field.
    pub fn get_song_id(&self) -> Uuid {
        self.song_id
    }
}
