use anyhow::Result;

use serde::Serialize;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use subd_macros::database_model;
use uuid::Uuid;

pub async fn find_random_instrumental(
    pool: &PgPool,
) -> Result<ai_songs::Model> {
    let res = sqlx::query_as!(
        ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
        WHERE lyric = '[Instrumental]'
        ORDER BY RANDOM()
        LIMIT 1
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(res)
}
pub async fn get_songs_for_user(
    pool: &PgPool,
    username: &str,
) -> Result<Vec<ai_songs::Model>> {
    let res = sqlx::query_as!(
        ai_songs::Model,
        r#"
        SELECT *
        FROM ai_songs
        WHERE username = $1
        ORDER BY created_at DESC
        "#,
        username
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
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

#[database_model]
pub mod ai_songs {
    use super::*;

    #[derive(Serialize)]
    pub struct Model {
        pub song_id: Uuid,
        pub title: String,
        pub tags: String,
        pub prompt: String,
        pub username: String,
        pub audio_url: String,
        pub gpt_description_prompt: String,

        pub lyric: Option<String>,

        #[serde(skip_serializing)]
        #[serde(with = "time::serde::rfc3339::option")]
        pub last_updated: Option<OffsetDateTime>,

        #[serde(skip_serializing)]
        #[serde(with = "time::serde::rfc3339::option")]
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
                ON CONFLICT (song_id)
                DO UPDATE SET gpt_description_prompt = $7
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

// ---------------------------------------------
//

/// Represents the type of vote: either "love" or "hate".
#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq, Default, Copy)]
#[sqlx(type_name = "vote_enum", rename_all = "lowercase")]
pub enum VoteType {
    #[default]
    Love,
    Hate,
}

#[database_model]
pub mod image_votes {
    use super::*;

    pub struct Model {
        pub user_id: Uuid,
        pub song_id: Uuid,
        pub image_name: String,
        pub vote_type: VoteType,
        pub voted_at: Option<OffsetDateTime>,
    }
}

pub async fn get_image_votes_or_default_with_extensions(
    pool: &PgPool,
    song_id: Uuid,
    image_names: Vec<String>,
) -> Result<Vec<(String, String, i64, i64)>, sqlx::Error> {
    // let image_names_without_ext: Vec<String> = image_names
    //     .iter()
    //     .map(|name| name.rsplit('.').next().unwrap_or(name).to_string())
    //     .collect();

    let res = sqlx::query!(
        r#"
        SELECT 
            image_name,
            COUNT(*) FILTER (WHERE vote_type = 'love') as love_count,
            COUNT(*) FILTER (WHERE vote_type = 'hate') as hate_count
        FROM image_votes
        WHERE image_name = ANY($1) AND song_id = $2
        GROUP BY image_name
        "#,
        &image_names,
        &song_id
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for image_name in image_names {
        let votes = res.iter().find(|row| row.image_name == image_name);
        result.push((
            image_name.clone(),
            image_name
                .split('.')
                .next()
                .unwrap_or(&image_name)
                .to_string(),
            votes.map_or(0, |v| v.love_count.unwrap_or(0)),
            votes.map_or(0, |v| v.hate_count.unwrap_or(0)),
        ));
    }

    Ok(result)
}

// Does this need to be a vec?
pub async fn get_all_image_votes_for_song(
    pool: &PgPool,
    song_id: Uuid,
) -> Result<Vec<(Uuid, String, i64, i64)>, sqlx::Error> {
    let res = sqlx::query!(
        r#"
            SELECT 
                image_name,
                COUNT(*) FILTER (WHERE vote_type = 'love') as love_count,
                COUNT(*) FILTER (WHERE vote_type = 'hate') as hate_count
            FROM image_votes
            WHERE song_id = $1
            GROUP BY image_name
            "#,
        song_id
    )
    .fetch_all(pool)
    .await?;

    Ok(res
        .into_iter()
        .map(|row| {
            (
                song_id,
                row.image_name,
                row.love_count.unwrap_or(0),
                row.hate_count.unwrap_or(0),
            )
        })
        .collect())
}

impl image_votes::Model {
    /// Saves a vote (love or hate) for an image associated with a song.
    pub async fn save(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO image_votes
            (user_id, song_id, image_name, vote_type)
            VALUES ($1, $2, $3, $4::vote_enum)
            ON CONFLICT (user_id, song_id, image_name)
            DO UPDATE SET vote_type = EXCLUDED.vote_type
            RETURNING
                user_id,
                song_id,
                image_name,
                vote_type as "vote_type: VoteType",
                voted_at
            "#,
            self.user_id,
            self.song_id,
            self.image_name,
            self.vote_type as VoteType,
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    pub async fn get_image_votes(
        pool: &PgPool,
        song_id: Uuid,
        image_name: &str,
    ) -> Result<(i64, i64), sqlx::Error> {
        let res = sqlx::query!(
            r#"
        SELECT 
            COUNT(*) FILTER (WHERE vote_type = 'love') as love_count,
            COUNT(*) FILTER (WHERE vote_type = 'hate') as hate_count
        FROM image_votes
        WHERE song_id = $1 AND image_name = $2
        "#,
            song_id,
            image_name
        )
        .fetch_one(pool)
        .await?;

        Ok((res.love_count.unwrap_or(0), res.hate_count.unwrap_or(0)))
    }
    pub async fn love_image(
        pool: &PgPool,
        user_id: Uuid,
        song_id: Uuid,
        image_name: &str,
    ) -> Result<Self, sqlx::Error> {
        Self::vote(pool, user_id, song_id, image_name, VoteType::Love).await
    }

    /// Casts a "hate" vote for the specified image.
    pub async fn hate_image(
        pool: &PgPool,
        user_id: Uuid,
        song_id: Uuid,
        image_name: &str,
    ) -> Result<Self, sqlx::Error> {
        Self::vote(pool, user_id, song_id, image_name, VoteType::Hate).await
    }

    /// Helper function to cast a vote (love or hate) for an image.
    pub async fn vote(
        pool: &PgPool,
        user_id: Uuid,
        song_id: Uuid,
        image_name: &str,
        vote_type: VoteType,
    ) -> Result<Self, sqlx::Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO image_votes
            (user_id, song_id, image_name, vote_type)
            VALUES ($1, $2, $3, $4::vote_enum)
            ON CONFLICT (user_id, song_id, image_name)
            DO UPDATE SET vote_type = EXCLUDED.vote_type
            RETURNING
                user_id,
                song_id,
                image_name,
                vote_type as "vote_type: VoteType",
                voted_at
            "#,
            user_id,
            song_id,
            image_name,
            vote_type as VoteType,
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }
}
