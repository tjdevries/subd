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

    pub async fn update_score(
        &self,
        pool: &PgPool,
        song_id: Uuid,
        user_id: Uuid,
        new_score: BigDecimal,
    ) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            UPDATE ai_song_vote
            SET score = $1
            WHERE song_id = $2 AND user_id = $3
            RETURNING
                song_id,
                user_id,
                good_song,
                score
            "#,
            new_score,
            song_id,
            user_id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn update_good_song(
        pool: &PgPool,
        song_id: Uuid,
        user_id: Uuid,
        new_good_song: bool,
    ) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            UPDATE ai_song_vote
            SET good_song = $1
            WHERE song_id = $2 AND user_id = $3
            RETURNING
                song_id,
                user_id,
                good_song,
                score
            "#,
            new_good_song,
            song_id,
            user_id,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn find_or_create_and_save_good_song(
        pool: &PgPool,
        song_id: Uuid,
        user_id: Uuid,
        good_song: bool,
    ) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO ai_song_vote (song_id, user_id, good_song)
            VALUES ($1, $2, $3)
            ON CONFLICT (song_id, user_id)
            DO UPDATE SET good_song = $3
            RETURNING
                song_id,
                user_id,
                good_song,
                score
            "#,
            song_id,
            user_id,
            good_song,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn find_or_create_and_save_score(
        pool: &PgPool,
        song_id: Uuid,
        user_id: Uuid,
        score: BigDecimal,
    ) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
                INSERT INTO ai_song_vote (song_id, user_id, score, good_song)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (song_id, user_id)
                DO UPDATE SET score = $3, good_song = $4
                RETURNING
                    song_id,
                    user_id,
                    good_song,
                    score
                "#,
            song_id,
            user_id,
            score,
            score >= BigDecimal::from(5),
        )
        .fetch_one(pool)
        .await?)
    }
}
