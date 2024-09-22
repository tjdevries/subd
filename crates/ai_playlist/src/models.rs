use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

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
    }
}

impl ai_songs::Model {
    #[allow(dead_code)]

    pub async fn save(&self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
                Self,
                r#"
                INSERT INTO ai_songs
                (song_id, title, tags, prompt, username, audio_url, gpt_description_prompt, lyric)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
                    created_at
                "#,
                self.song_id,
                self.title,
                self.tags,
                self.prompt,
                self.username,
                self.audio_url,
                self.gpt_description_prompt,
                self.lyric,
            )
            .fetch_one(pool)
            .await?)
    }

    /// Returns the `song_id` field.
    pub fn get_song_id(&self) -> Uuid {
        self.song_id
    }

    /// Returns a reference to the `title` field.
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// Returns a reference to the `tags` field.
    pub fn get_tags(&self) -> &str {
        &self.tags
    }

    /// Returns a reference to the `prompt` field.
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }

    /// Returns a reference to the `username` field.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns a reference to the `audio_url` field.
    pub fn get_audio_url(&self) -> &str {
        &self.audio_url
    }

    /// Returns a reference to the `lyric` field.
    // pub fn get_lyric(&self) -> &str {
    //     &self.lyric
    // }

    /// Returns a reference to the `gpt_description_prompt` field.
    pub fn get_gpt_description_prompt(&self) -> &str {
        &self.gpt_description_prompt
    }
}

pub async fn find_by_id(
    pool: &sqlx::PgPool,
    song_id: Uuid,
) -> Result<ai_songs::Model> {
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
