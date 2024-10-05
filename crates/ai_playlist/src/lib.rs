use anyhow::anyhow;
use anyhow::Result;
use sqlx::{types::time::OffsetDateTime, Error, PgPool};
use uuid::Uuid;

pub mod models;

pub async fn get_unplayed_songs(
    pool: &PgPool,
) -> Result<Vec<models::ai_songs::Model>> {
    let res = sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_song_playlist
        JOIN ai_songs ON ai_song_playlist.song_id = ai_songs.song_id
        WHERE ai_song_playlist.played_at IS NULL
        ORDER BY ai_song_playlist.created_at ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(res)
}

pub async fn total_ai_songs(pool: &sqlx::PgPool) -> Result<i64> {
    let record = sqlx::query!(
        "
        SELECT COUNT(*) as count
        FROM ai_songs
        ",
    )
    .fetch_one(pool)
    .await?;
    record.count.ok_or(anyhow!("Error on ai_songs count"))
}

pub async fn find_song_by_id(
    pool: &PgPool,
    song_id: &str,
) -> Result<models::ai_songs::Model> {
    let uuid = Uuid::parse_str(song_id)
        .map_err(|_| anyhow!("Couldn't parse song_id"))?;
    sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
        WHERE song_id = $1
        "#,
        uuid
    )
    .fetch_optional(pool)
    .await?
    .ok_or(anyhow!("Couldn't find song"))
}

pub async fn find_random_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model, Error> {
    sqlx::query_as!(
        models::ai_songs::Model,
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

// Fetches a random song ID from the database.
pub async fn find_random_song_id(pool: &PgPool) -> Result<Uuid, Error> {
    sqlx::query_scalar!(
        r#"
        SELECT song_id
        FROM ai_songs
        ORDER BY RANDOM()
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
}

// Fetches the last played song ID from the playlist.
pub async fn find_last_played_song(pool: &PgPool) -> Result<Uuid, Error> {
    sqlx::query_scalar!(
        r#"
        SELECT song_id
        FROM ai_song_playlist
        WHERE played_at IS NOT NULL
        ORDER BY played_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
}

// Fetches the oldest unplayed song from the playlist.
pub async fn find_oldest_unplayed_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model, Error> {
    sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_song_playlist
        JOIN ai_songs ON ai_song_playlist.song_id = ai_songs.song_id
        WHERE ai_song_playlist.played_at IS NULL
        ORDER BY ai_song_playlist.created_at ASC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
}

pub async fn count_unplayed_songs(pool: &PgPool) -> Result<i64, Error> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM ai_song_playlist
        WHERE played_at IS NULL AND stopped_at IS NULL
        "#
    )
    .fetch_one(pool)
    .await?;

    match count {
        Some(count) => Ok(count),
        None => Ok(0),
    }
}
pub async fn add_song_to_playlist(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<models::ai_playlist::Model, Error> {
    let playlist_entry = models::ai_playlist::Model {
        playlist_id: Uuid::new_v4(),
        song_id,
        created_at: Some(OffsetDateTime::now_utc()),
        played_at: None,
        stopped_at: None,
    };

    playlist_entry.save(pool).await
}

// Marks a song as played in the playlist.
pub async fn mark_song_as_downloaded(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE ai_songs
        SET downloaded = true 
        WHERE song_id = $1
        "#,
        song_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Marks a song as played in the playlist.
pub async fn mark_song_as_played(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE ai_song_playlist
        SET played_at = NOW()
        WHERE song_id = $1
        "#,
        song_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Marks all currently playing songs as stopped.
pub async fn mark_songs_as_stopped(pool: &PgPool) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE ai_song_playlist
        SET stopped_at = NOW()
        WHERE stopped_at IS NULL AND played_at IS NOT NULL
        "#
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Gets the currently playing song, or the oldest unplayed song if none is playing.
pub async fn get_current_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model, Error> {
    if let Some(song) = sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_song_playlist
        JOIN ai_songs ON ai_song_playlist.song_id = ai_songs.song_id
        WHERE ai_song_playlist.played_at IS NOT NULL
          AND ai_song_playlist.stopped_at IS NULL
        ORDER BY ai_song_playlist.played_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    {
        Ok(song)
    } else {
        // Return the oldest unplayed song if no song is currently playing.
        find_oldest_unplayed_song(pool).await
    }
}
pub async fn update_song_tags(
    pool: &PgPool,
    song_id: Uuid,
    new_tags: String,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE ai_songs
        SET tags = $1
        WHERE song_id = $2
        "#,
        new_tags,
        song_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
pub async fn find_songs_without_tags(
    pool: &PgPool,
) -> Result<Vec<models::ai_songs::Model>, Error> {
    let songs = sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
        WHERE tags IS NULL OR tags = ''
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(songs)
}
// Fetches the last played songs up to a specified limit.
pub async fn find_last_played_songs(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<models::ai_songs::Model>, Error> {
    let songs = sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_song_playlist
        JOIN ai_songs ON ai_song_playlist.song_id = ai_songs.song_id
        WHERE ai_song_playlist.played_at IS NOT NULL
        ORDER BY ai_song_playlist.played_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(songs)
}

// Fetches songs by a specific user.
pub async fn find_songs_by_user(
    pool: &PgPool,
    username: &str,
) -> Result<Vec<models::ai_songs::Model>, Error> {
    let songs = sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
        WHERE username = $1
        "#,
        username
    )
    .fetch_all(pool)
    .await?;

    Ok(songs)
}

// Fetches the next song to play from the playlist.
pub async fn find_next_song_to_play(
    pool: &PgPool,
) -> Result<models::ai_songs::Model, Error> {
    sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT ai_songs.*
        FROM ai_song_playlist
        JOIN ai_songs ON ai_song_playlist.song_id = ai_songs.song_id
        WHERE ai_song_playlist.played_at IS NULL
        ORDER BY ai_song_playlist.created_at ASC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?
    .ok_or(Error::RowNotFound)
}

pub async fn delete_song(pool: &PgPool, song_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"
        DELETE FROM ai_song_playlist
        WHERE song_id = $1
        "#,
        song_id
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM ai_songs
        WHERE song_id = $1
        "#,
        song_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
