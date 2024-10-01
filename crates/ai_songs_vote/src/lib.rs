pub mod models;
use ai_playlist;
use anyhow::Result;
use sqlx;
use sqlx::types::BigDecimal;

use sqlx::{postgres::PgPool, FromRow};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct AiSongRanking {
    pub song_id: Uuid,
    pub title: String,
    pub avg_score: f64,
}

pub async fn get_top_songs(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<AiSongRanking>> {
    let songs = sqlx::query_as::<_, AiSongRanking>(
        r#"
        SELECT
            s.song_id,
            s.title,
            CAST(AVG(v.score) AS DOUBLE PRECISION) AS avg_score
        FROM
            ai_songs s
        JOIN
            ai_songs_vote v ON s.song_id = v.song_id
        GROUP BY
            s.song_id, s.title
        ORDER BY
            avg_score DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(songs)
}

pub async fn vote_for_current_song_with_score(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    score: f64,
) -> Result<()> {
    let current_song = ai_playlist::get_current_song(pool).await?;
    let score = BigDecimal::try_from(score)?;

    let _ = models::find_or_create_and_save_score(
        pool,
        current_song.song_id,
        user_id,
        score,
    )
    .await;
    Ok(())
}
