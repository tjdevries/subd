pub mod models;
use ai_playlist;
use anyhow::anyhow;
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

pub async fn total_votes_by_id(pool: &PgPool, song_id: Uuid) -> Result<i64> {
    let record = sqlx::query!(
        "
        SELECT COUNT(*) as count
        FROM ai_songs_vote
        WHERE song_id = $1
        ",
        song_id
    )
    .fetch_one(pool)
    .await?;
    record
        .count
        .ok_or(anyhow!("Error on ai_songs by song_id count"))
}

pub async fn total_votes(pool: &PgPool) -> Result<i64> {
    let record = sqlx::query!(
        "
        SELECT COUNT(*) as count
        FROM ai_songs_vote
        ",
    )
    .fetch_one(pool)
    .await?;
    record.count.ok_or(anyhow!("Error on ai_songs count"))
}

pub async fn get_average_score(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<AiSongRanking> {
    let ranking = sqlx::query_as::<_, AiSongRanking>(
        r#"
        SELECT
            s.song_id,
            s.title,
            CAST(AVG(v.score) AS DOUBLE PRECISION) AS avg_score
        FROM
            ai_songs s
        JOIN
            ai_songs_vote v ON s.song_id = v.song_id
        WHERE
            s.song_id = $1
        GROUP BY
            s.song_id, s.title
        ORDER BY
            avg_score DESC
        LIMIT 1
        "#,
    )
    .bind(song_id)
    .fetch_one(pool)
    .await?;
    Ok(ranking)
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
    //
    //    SELECT
    //            s.song_id,
    //            s.title,
    //            CAST(AVG(v.score) AS DOUBLE PRECISION) AS avg_score,
    //            CAST(SUM(v.score) AS DOUBLE PRECISION) AS total_score,
    //            (COUNT(*) / 10) AS multipler
    //        FROM
    //            ai_songs s
    //        JOIN
    //            ai_songs_vote v ON s.song_id = v.song_id
    //        GROUP BY
    //            s.song_id, s.title
    //        ORDER BY
    //            avg_score DESC
    //;

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
