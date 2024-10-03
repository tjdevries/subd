use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

// This should maybe in lib.rs
pub async fn count_of_ai_songs(_pool: &sqlx::PgPool) -> Result<u64> {
    Ok(0)
}

#[database_model]
pub mod ai_songs {
    use super::*;

    pub struct Model {
        pub song_id: Uuid,
        pub title: String,
        pub tags: String,
        pub prompt: String,
        pub username: String,
        pub audio_url: String,
        pub gpt_description_prompt: String,
        pub lyric: Option<String>,
        pub last_updated: Option<OffsetDateTime>,
        pub created_at: Option<OffsetDateTime>,

        // This has a default of false in the DB, so I think this could be optional
        // need to double check migration and actual tables
        pub downloaded: bool,
    }
}

impl ai_songs::Model {
    #[allow(dead_code)]

    pub async fn save(&self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
                Self,
                r#"
                INSERT INTO ai_songs
                (song_id, title, tags, prompt, username, audio_url, gpt_description_prompt, lyric, downloaded)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING 
                    song_id, 
                    title, 
                    tags, 
                    prompt, 
                    username, 
                    audio_url, 
                    gpt_description_prompt, 
                    lyric, 
                    last_updated, 
                    created_at,
                    downloaded
                "#,
                self.song_id,
                self.title,
                self.tags,
                self.prompt,
                self.username,
                self.audio_url,
                self.gpt_description_prompt,
                self.lyric,
                self.downloaded,
            )
            .fetch_one(pool)
            .await?)
    }
}

// This is confusing because you're not sure if its for ai_playlist of ai_songs
pub async fn find_by_id(
    pool: &sqlx::PgPool,
    song_id: Uuid,
) -> Result<ai_songs::Model> {
    let res =
        sqlx::query!("SELECT * FROM ai_songs WHERE song_id = $1", song_id)
            .fetch_one(pool)
            .await?;

    // TODO: it seems wierd we can't just return a single object
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
        downloaded: res.downloaded,
    };
    Ok(model)
}

#[database_model]
pub mod ai_playlist {
    use super::*;

    pub struct Model {
        pub playlist_id: Uuid,
        pub song_id: Uuid,
        pub created_at: Option<OffsetDateTime>,
        pub played_at: Option<OffsetDateTime>,
        pub stopped_at: Option<OffsetDateTime>,
    }
}

impl ai_playlist::Model {
    #[allow(dead_code)]
    pub async fn save(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
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
        .await
    }
}
