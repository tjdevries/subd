use anyhow::Result;
use reqwest::Client as ReqwestClient;
use subd_types::{UserID, UserPlatform};
use twitch_api::helix::subscriptions::GetBroadcasterSubscriptionsRequest;
use twitch_api::helix::HelixClient;
use twitch_oauth2::UserToken;

pub async fn save_twitch_message(
    pool: &sqlx::PgPool,
    user_id: &UserID,
    platform: UserPlatform,
    message: &str,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO user_messages (user_id, platform, contents)
                VALUES ( $1, $2, $3 )"#,
        user_id.0,
        platform as _,
        message
    )
    .execute(pool)
    .await?;

    Ok(())
}

// TODO: I think this is where we might be failing
pub async fn upsert_twitch_user(
    pool: &sqlx::PgPool,
    twitch_user_id: &subd_types::TwitchUserID,
    twitch_user_login: &str,
) -> Result<UserID> {
    // TODO: We should create one transaction for this...

    match sqlx::query!(
        "SELECT user_id FROM twitch_users WHERE twitch_user_id = $1",
        twitch_user_id.0
    )
    .fetch_optional(pool)
    .await?
    {
        Some(twitch_user) => Ok(UserID(twitch_user.user_id)),
        None => {
            let user_id = create_new_user(pool).await?;

            sqlx::query!(
                "INSERT INTO twitch_users (user_id, twitch_user_id, login, display_name)
                VALUES($1, $2, $3, $4)",
                user_id.0,
                twitch_user_id.0,
                twitch_user_login,
                twitch_user_login
            )
            .execute(pool)
            .await?;

            Ok(user_id)
        }
    }
}

// Not sure why this req_get failed
// pub async fn get_twitch_sub_count<'a>(
//     client: &HelixClient<'a, ReqwestClient>,
//     token: UserToken,
// ) -> usize {
//     let req = GetBroadcasterSubscriptionsRequest::broadcaster_id(
//         token.user_id.clone(),
//     );
//
//     let response = client
//         .req_get(req, &token)
//         .await
//         .expect("Error Fetching Twitch Subs");
//
//     response.total.unwrap() as usize
// }

pub async fn create_new_user(conn: &sqlx::PgPool) -> Result<UserID> {
    let x = sqlx::query!("INSERT INTO users DEFAULT VALUES RETURNING user_id")
        .fetch_one(conn)
        .await?;

    Ok(UserID(x.user_id))
}
