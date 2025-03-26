use super::*;
use anyhow::Result;
use sqlx::PgPool;
use test_tag::tag;
use uuid::Uuid;

#[tokio::test]
#[tag(database)]
async fn test_ai_song_functions() {
    // TODO: the fact we are using multiple DBs this is a problem
    let pool = subd_db::get_test_db_pool().await;
    delete_all_ai_songs_and_playlist(&pool).await.unwrap();
    let count = total_ai_songs(&pool).await.unwrap();
    assert_eq!(count, 0, "ok");

    // they say there is default, but it ain't working
    let m = models::ai_songs::Model {
        ..Default::default()
    };

    // I should add Builder Pattern????
    // let m = models::ai_songs::Model::new();

    assert_eq!(m.title, "");

    // This is creating 2?
    // or 2 or running at the same time?
    let res = m.save(&pool).await;
    assert!(res.is_ok());
    let count = total_ai_songs(&pool).await.unwrap();
    // since tests are async counts are hard
    assert!(count > 0);
}

#[tokio::test]
#[tag(database)]
async fn test_ai_song_creation() {
    let pool = subd_db::get_test_db_pool().await;
    delete_all_ai_songs_and_playlist(&pool).await.unwrap();

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

async fn delete_all_ai_songs_and_playlist(pool: &PgPool) -> Result<()> {
    let db_name = sqlx::query!("SELECT current_database()")
        .fetch_one(pool)
        .await?
        .current_database
        .unwrap_or_default();

    if !db_name.starts_with("test_") {
        return Err(anyhow::anyhow!(
            "This operation is only allowed on test databases"
        ));
    }

    sqlx::query!("DELETE FROM ai_song_playlist")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM ai_songs").execute(pool).await?;
    Ok(())
}
