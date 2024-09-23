use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::PgPool;
use uuid::Uuid;

pub mod models;

pub async fn find_oldest_unplayed_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model> {
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
        let song = models::ai_songs::Model {
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
) -> Result<models::ai_playlist::Model, sqlx::Error> {
    let playlist_entry = models::ai_playlist::Model {
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

//pub async fn mark_song_as_stopped(
//    pool: &PgPool,
//    song_id: Uuid,
//) -> Result<(), sqlx::Error> {
//    sqlx::query!(
//        r#"
//        UPDATE ai_playlist
//        SET stopped_at = NOW()
//        WHERE song_id = $1
//        "#,
//        song_id
//    )
//    .execute(pool)
//    .await?;
//    Ok(())
//}

pub async fn get_current_song(
    pool: &PgPool,
) -> Result<models::ai_songs::Model> {
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
        let song = models::ai_songs::Model {
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
) -> Result<Vec<models::ai_songs::Model>, sqlx::Error> {
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
        .map(|res| models::ai_songs::Model {
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
) -> Result<Vec<models::ai_songs::Model>, sqlx::Error> {
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
        .map(|res| models::ai_songs::Model {
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

//pub async fn find_by_id(
//    pool: &PgPool,
//    song_id: Uuid,
//) -> Result<ai_songs::Model, sqlx::Error> {
//    let res =
//        sqlx::query!("SELECT * FROM ai_songs WHERE song_id = $1", song_id)
//            .fetch_one(pool)
//            .await?;
//
//    let model = ai_songs::Model {
//        song_id,
//        title: res.title,
//        tags: res.tags,
//        prompt: res.prompt,
//        username: res.username,
//        audio_url: res.audio_url,
//        lyric: res.lyric,
//        gpt_description_prompt: res.gpt_description_prompt,
//        last_updated: res.last_updated,
//        created_at: res.created_at,
//    };
//    Ok(model)
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ai_songs;
    use anyhow::Result;
    use test_tag::tag;

    // This is scary if you run this on the "Prod" database
    async fn delete_all_ai_songs(pool: &PgPool) -> Result<()> {
        sqlx::query!("DELETE FROM ai_songs")
            .execute(pool)
            .await
            .unwrap();
        Ok(())
    }

    #[tokio::test]
    #[tag(database)]
    async fn test_ai_song_creation() {
        let pool = subd_db::get_test_db_pool().await;
        let _ = delete_all_ai_songs(&pool).await;

        let ai_song = ai_songs::Model::new(
            Uuid::parse_str("d7d9d6d5-9b4c-4b2f-8d8e-2d5f6b3b2b4f").unwrap(),
            "title".to_string(),
            "tags".to_string(),
            "prompt".to_string(),
            "username".to_string(),
            "audio_url".to_string(),
            "gpt_description_prompt".to_string(),
            None,
            None,
            None,
        );

        ai_song.save(&pool).await.unwrap();
        let res = find_songs_by_user(&pool, "username").await.unwrap();
        assert_eq!(res[0].title, "title");

        // this is requires to be on the AI playlist
        // Find newest_song
        let result = find_last_played_songs(&pool, 1).await.unwrap();
        assert_eq!(result.len(), 1);
        println!("OK This does work");
        result.iter().for_each(|song| {
            println!("{:?}", song);
        });
    }
}
