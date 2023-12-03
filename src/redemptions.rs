use anyhow::Result;
use sqlx::PgPool;
use subd_macros::database_model;

// #[database_model]
// pub mod twitch_stream_state {
//     use super::*;
//
//     pub struct Model {
//         pub sub_only_tts: bool,
//         pub explicit_soundeffects: bool,
//         pub implicit_soundeffects: bool,
//         pub global_voice: bool,
//     }
// }
//
// impl twitch_stream_state::Model {
//     #[allow(dead_code)]
//
//     pub async fn save(self, pool: &PgPool) -> Result<Self> {
//         Ok(sqlx::query_as!(
//             Self,
//             r#"
//             INSERT INTO twitch_stream_state
//             (sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice)
//             VALUES ( $1, $2, $3, $4)
//             RETURNING sub_only_tts, explicit_soundeffects, implicit_soundeffects, global_voice
//         "#,
//             true,
//             true,
//             true,
//             false,
//         )
//         .fetch_one(pool)
//         .await?)
//     }
// }
//
// pub async fn turn_off_global_voice(pool: &PgPool) -> Result<()> {
//     let _res =
//         sqlx::query!("UPDATE twitch_stream_state SET global_voice = $1", false)
//             .execute(pool)
//             .await?;
//
//     Ok(())
// }
//
