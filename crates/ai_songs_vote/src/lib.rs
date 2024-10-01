pub mod models;
use ai_playlist;
use anyhow::Result;
use sqlx;
use sqlx::types::BigDecimal;
// use sqlx::Uuid;
use uuid::Uuid;

pub async fn vote_for_current_song_with_score(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    score: f64,
) -> Result<()> {
    let current_song = ai_playlist::get_current_song(pool).await?;
    let score = BigDecimal::try_from(score)?;

    let _ = models::find_or_create_and_save_score(
        pool,
        current_song.song_id,
        user_id,
        score,
    )
    .await;
    Ok(())
}
