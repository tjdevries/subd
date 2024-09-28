use anyhow::anyhow;
use anyhow::Result;
use sqlx::{types::time::OffsetDateTime, Error, PgPool};
use uuid::Uuid;

pub mod models;

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

// Fetches a random song from the database.
pub async fn find_random_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model, Error> {
    sqlx::query_as!(
        models::ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
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

// Adds a song to the playlist.
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
mod tests {
    use super::*;
    use crate::models::ai_songs;
    use anyhow::Result;
    use test_tag::tag;

    // Deletes all songs and playlist entries. Use with caution.
    async fn delete_all_ai_songs_and_playlist(pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM ai_song_playlist")
            .execute(pool)
            .await?;
        sqlx::query!("DELETE FROM ai_songs").execute(pool).await?;
        Ok(())
    }

    #[tokio::test]
    #[tag(database)]
    async fn test_ai_song_creation() {
        let pool = subd_db::get_test_db_pool().await;
        let _ = delete_all_ai_songs_and_playlist(&pool).await.unwrap();

        // This must not be working here

        let fake_uuid = Uuid::new_v4();
        let ai_song = ai_songs::Model::new(
            fake_uuid,
            "title".into(),
            "tags".into(),
            "prompt".into(),
            "username".into(),
            "audio_url".into(),
            "gpt_description_prompt".into(),
            None,
            None,
            None,
            false,
        );

        ai_song.save(&pool).await.unwrap();
        let res = find_songs_by_user(&pool, "username").await.unwrap();
        assert_eq!(res[0].title, "title");

        // This is failing for some reason
        add_song_to_playlist(&pool, fake_uuid).await.unwrap();
        let result = find_last_played_songs(&pool, 1).await.unwrap();
        assert!(result.is_empty());

        let next_song = find_next_song_to_play(&pool).await.unwrap();
        assert_eq!(next_song.title, "title");

        mark_song_as_played(&pool, fake_uuid).await.unwrap();

        let next_song = find_next_song_to_play(&pool).await;
        assert!(next_song.is_err());

        let last_song_uuid = find_last_played_song(&pool).await.unwrap();
        assert_eq!(last_song_uuid, fake_uuid);

        mark_song_as_played(&pool, fake_uuid).await.unwrap();
        let next_song = find_next_song_to_play(&pool).await;
        assert!(next_song.is_err());

        // Add multiple unplayed songs
        let aaaa_uuid = Uuid::new_v4();
        let ai_song = ai_songs::Model::new(
            aaaa_uuid,
            "title".into(),
            "tags".into(),
            "prompt".into(),
            "username".into(),
            "audio_url".into(),
            "gpt_description_prompt".into(),
            None,
            None,
            None,
            false,
        );
        ai_song.save(&pool).await.unwrap();
        add_song_to_playlist(&pool, aaaa_uuid).await.unwrap();

        let bbbb_uuid = Uuid::new_v4();
        let ai_song = ai_songs::Model::new(
            bbbb_uuid,
            "title".into(),
            "tags".into(),
            "prompt".into(),
            "username".into(),
            "audio_url".into(),
            "gpt_description_prompt".into(),
            None,
            None,
            None,
            false,
        );
        ai_song.save(&pool).await.unwrap();
        add_song_to_playlist(&pool, bbbb_uuid).await.unwrap();

        let next_song = find_next_song_to_play(&pool).await.unwrap();
        assert_eq!(next_song.song_id, aaaa_uuid);

        mark_song_as_played(&pool, aaaa_uuid).await.unwrap();
        let next_song = find_next_song_to_play(&pool).await.unwrap();
        assert_eq!(next_song.song_id, bbbb_uuid);
    }
}
