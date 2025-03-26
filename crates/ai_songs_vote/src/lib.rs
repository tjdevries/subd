use anyhow::anyhow;
use anyhow::Result;
use serde::Serialize;
use sqlx::types::BigDecimal;
use sqlx::Error;
use sqlx::{postgres::PgPool, FromRow};
use uuid::Uuid;

pub mod models;

#[derive(Serialize, Debug, FromRow)]
pub struct AiSongRanking {
    pub song_id: Uuid,
    pub title: String,
    pub avg_score: f64,
}

#[derive(Serialize)]
pub struct Stats {
    pub ai_songs_count: i64,
    pub ai_votes_count: i64,
    pub unplayed_songs_count: i64,
}

#[derive(Serialize)]
pub struct CurrentSongInfo {
    pub current_song: Option<ai_playlist::models::ai_songs::Model>,
    pub votes_count: i64,
}

pub async fn get_current_song_info(pool: &PgPool) -> Result<CurrentSongInfo> {
    let current_song = ai_playlist::get_current_song(pool).await.ok();
    let votes_count = match current_song.as_ref() {
        Some(song) => total_votes_by_id(pool, song.song_id).await.unwrap_or(0),
        None => 0,
    };
    Ok(CurrentSongInfo {
        current_song,
        votes_count,
    })
}
pub async fn fetch_stats(pool: &PgPool) -> Result<Stats> {
    let ai_songs_count = ai_playlist::total_ai_songs(pool).await.unwrap_or(0);
    let ai_votes_count = total_votes(pool).await.unwrap_or(0);
    let unplayed_songs_count =
        ai_playlist::count_unplayed_songs(pool).await.unwrap_or(0);
    Ok(Stats {
        ai_songs_count,
        ai_votes_count,
        unplayed_songs_count,
    })
}

pub async fn get_users_with_song_count(
    pool: &PgPool,
) -> Result<Vec<(String, Option<i64>)>> {
    let res = sqlx::query!(
        r#"
        SELECT username, COUNT(*) as song_count
        FROM ai_songs
        GROUP BY username
        ORDER BY song_count DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(res
        .into_iter()
        .map(|row| (row.username, row.song_count))
        .collect())
}

pub async fn get_users_with_song_count_and_avg_score(
    pool: &PgPool,
) -> Result<Vec<(String, i64, f64)>> {
    let res = sqlx::query!(
        r#"
        SELECT 
            a.username, 
            COUNT(*) as song_count, 
            CAST(AVG(v.avg_score) AS DOUBLE PRECISION) AS avg_score
        FROM ai_songs a
        LEFT JOIN (
            SELECT song_id, AVG(score) as avg_score
            FROM ai_songs_vote
            GROUP BY song_id
        ) v ON a.song_id = v.song_id
            GROUP BY a.username
            ORDER BY song_count DESC, avg_score DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(res
        .into_iter()
        .map(|row| {
            (
                row.username,
                row.song_count.unwrap_or(0),
                row.avg_score.unwrap_or(0.0),
            )
        })
        .collect())
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

pub async fn find_random_song(
    pool: &PgPool,
) -> Result<ai_playlist::models::ai_songs::Model, Error> {
    sqlx::query_as!(
        ai_playlist::models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_songs
        LEFT JOIN ai_song_playlist ON ai_songs.song_id = ai_song_playlist.song_id
        WHERE ai_song_playlist.played_at IS NULL
           OR ai_song_playlist.played_at < NOW() - INTERVAL '1 hour'
        ORDER BY RANDOM()
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
}

pub async fn get_random_high_rated_song(
    pool: &PgPool,
) -> Result<AiSongRanking> {
    let song = sqlx::query_as::<_, AiSongRanking>(
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
        HAVING
            AVG(v.score) > 9.0
        ORDER BY
            RANDOM()
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(song)
}

// We might want a way to get dynamicd average score limits
pub async fn get_random_high_rated_recent_song(
    pool: &PgPool,
) -> Result<AiSongRanking, Error> {
    sqlx::query_as::<_, AiSongRanking>(
        r#"
        SELECT
            s.song_id,
            s.title,
            CAST(AVG(v.score) AS DOUBLE PRECISION) AS avg_score
        FROM
            ai_songs s
        JOIN
            ai_songs_vote v ON s.song_id = v.song_id
        LEFT JOIN
            ai_song_playlist p ON s.song_id = p.song_id
        WHERE
            p.played_at IS NULL OR p.played_at < NOW() - INTERVAL '1 hour'
        GROUP BY
            s.song_id, s.title
        HAVING
            AVG(v.score) > 9.0
        ORDER BY
            RANDOM()
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
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
