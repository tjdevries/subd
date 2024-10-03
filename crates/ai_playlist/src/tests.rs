use super::*;
use anyhow::Result;
use sqlx::PgPool;
use test_tag::tag;
use uuid::Uuid;

#[tokio::test]
#[tag(database)]
async fn test_ai_song_functions() {
    let pool = subd_db::get_test_db_pool().await;
    let count = total_ai_songs(&pool).await.unwrap();
    assert_eq!(count, 0);

    // Now I need to create an ai_songs
}

#[tokio::test]
#[tag(database)]
async fn test_ai_song_creation() {
    let pool = subd_db::get_test_db_pool().await;
    let _ = delete_all_ai_songs_and_playlist(&pool).await.unwrap();

    // This must not be working here

    let fake_uuid = Uuid::new_v4();
    let ai_song = models::ai_songs::Model::new(
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

    let res = ai_song.save(&pool).await.unwrap();
    assert_eq!(res.title, "title");

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
    let ai_song = models::ai_songs::Model::new(
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
    let ai_song = models::ai_songs::Model::new(
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

// Helper Methods =======================================================

// Deletes all songs and playlist entries. Use with caution.
async fn delete_all_ai_songs_and_playlist(pool: &PgPool) -> Result<()> {
    sqlx::query!("DELETE FROM ai_song_playlist")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM ai_songs").execute(pool).await?;
    Ok(())
}
