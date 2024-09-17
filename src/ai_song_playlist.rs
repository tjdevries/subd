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
        pub stopped_at: Option<OffsetDateTime>,
    }
}

impl ai_song_playlist::Model {
    #[allow(dead_code)]
    pub async fn save(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO ai_song_playlist
            (playlist_id, song_id, created_at, played_at, stopped_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                playlist_id,
                song_id,
                created_at,
                played_at,
                stopped_at
            "#,
            self.playlist_id,
            self.song_id,
            self.created_at,
            self.played_at,
            self.stopped_at
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn find_by_id(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<ai_songs::Model, sqlx::Error> {
    let res =
        sqlx::query!("SELECT * FROM ai_songs WHERE song_id = $1", song_id)
            .fetch_one(pool)
            .await?;

    let model = ai_songs::Model {
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

pub async fn find_oldest_unplayed_song(
    pool: &PgPool,
) -> Result<ai_songs::Model> {
    let res = sqlx::query!(
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
    .await?;

    if let Some(res) = res {
        let song = ai_songs::Model {
            song_id: res.song_id,
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
        Ok(song)
    } else {
        Err(anyhow::anyhow!("No unplayed songs found"))
    }
}

pub async fn add_song_to_playlist(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<ai_song_playlist::Model, sqlx::Error> {
    let playlist_entry = ai_song_playlist::Model {
        playlist_id: Uuid::new_v4(),
        song_id,
        created_at: Some(OffsetDateTime::now_utc()),
        played_at: None,
        stopped_at: None,
    };

    playlist_entry.save(pool).await
}

pub async fn mark_song_as_played(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<(), sqlx::Error> {
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

// I should be able to just mark the songs that's stopped as stopped
pub async fn mark_songs_as_stopped(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE ai_song_playlist
        SET stopped_at = NOW()
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_song_as_stopped(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE ai_song_playlist
        SET stopped_at = NOW()
        WHERE song_id = $1
        "#,
        song_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_current_song(pool: &PgPool) -> Result<ai_songs::Model> {
    let res = sqlx::query!(
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
    .await?;

    if let Some(res) = res {
        let song = ai_songs::Model {
            song_id: res.song_id,
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
        Ok(song)
    } else {
        // Not sure what this should be
        // If no song has been played yet, return the oldest unplayed song
        Ok(find_oldest_unplayed_song(pool).await?)
    }
}

pub async fn find_last_played_songs(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<ai_songs::Model>, sqlx::Error> {
    let records = sqlx::query!(
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

    let songs = records
        .into_iter()
        .map(|res| ai_songs::Model {
            song_id: res.song_id,
            title: res.title,
            tags: res.tags,
            prompt: res.prompt,
            username: res.username,
            audio_url: res.audio_url,
            lyric: res.lyric,
            gpt_description_prompt: res.gpt_description_prompt,
            last_updated: res.last_updated,
            created_at: res.created_at,
        })
        .collect();

    Ok(songs)
}

pub async fn find_songs_by_user(
    pool: &PgPool,
    username: &str,
) -> Result<Vec<ai_songs::Model>, sqlx::Error> {
    let records = sqlx::query!(
        r#"
        SELECT *
        FROM ai_songs
        WHERE username = $1
        "#,
        username
    )
    .fetch_all(pool)
    .await?;

    let songs = records
        .into_iter()
        .map(|res| ai_songs::Model {
            song_id: res.song_id,
            title: res.title,
            tags: res.tags,
            prompt: res.prompt,
            username: res.username,
            audio_url: res.audio_url,
            lyric: res.lyric,
            gpt_description_prompt: res.gpt_description_prompt,
            last_updated: res.last_updated,
            created_at: res.created_at,
        })
        .collect();

    Ok(songs)
}
