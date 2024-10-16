use super::*;
use ai_playlist::models::ai_songs;
use uuid::Uuid;

#[tokio::test]
async fn test_highest_file_number() {
    let uuid_str = "0833d255-607f-4b74-bea9-4818f032140a";
    let id = Uuid::parse_str(uuid_str).unwrap();
    let music_video_folder = format!("../../tmp/music_videos/{}", id);
    let highest_number = std::fs::read_dir(music_video_folder)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .filter_map(|name| name.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    println!("Highest Number: {}", highest_number);
    assert_eq!(1727998927, highest_number);
}

#[ignore]
#[tokio::test]
async fn test_create_music_video() {
    let pool = subd_db::get_test_db_pool().await;

    let fake_uuid = Uuid::new_v4();
    let ai_song = ai_songs::Model::new(
        fake_uuid,
        "title".into(),
        "tags".into(),
        "prompt".into(),
        "username".into(),
        "audio_url".into(),
        "gpt_description_prompt".into(),
        Some("Lyrics Hooray!".to_string()),
        None,
        None,
        false,
    );

    ai_song.save(&pool).await.unwrap();
    let id = format!("{}", fake_uuid);
    let _res = create_music_video(&pool, id).await.unwrap();
    // OK
}
