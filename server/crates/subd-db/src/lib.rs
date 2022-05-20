#![allow(dead_code)]

use anyhow::Result;
use sqlx::{Connection, SqliteConnection};

pub type UserID = i64;
pub struct User {
    pub id: UserID,
    pub twitch_user: Option<String>,
    pub github_user: Option<String>,
    // last_updated: chrono::Utc,
}

// static CONNECTIONS: Lazy<Pool<Sqlite>> = Lazy::new(|| {
//     let options = SqliteConnectOptions::new()
//         .filename(std::env::var("DATABASE_URL").expect("Must have passed filename"));
//
//     // SqlitePool::connect("sql
//     let pool = SqlitePool::connect_lazy_with(options);
//
//     // TODO: Can I do this synchronously?
//     // sqlx::migrate!().run(&pool).await?;
//
//     pool
// });

pub async fn get_handle() -> SqliteConnection {
    SqliteConnection::connect("sqlite:/home/tjdevries/git/subd/server/subd.db")
        .await
        .expect("To connect to the database")
}

pub async fn set_github_user_for_user(
    conn: &mut SqliteConnection,
    user: &UserID,
    github_user: &str,
) -> Result<()> {
    // Make sure that the user exists
    sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES ( ?1 )", user)
        .execute(&mut *conn)
        .await?;

    // Update user to have twitch_user name
    sqlx::query!(
        "UPDATE users SET github_user = ?2 WHERE id = ?1",
        user,
        github_user
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

// async fn set_twitch_user_for_user(
//     conn: &mut SqliteConnection,
//     user: &UserID,
//     twitch_user: &str,
// ) -> anyhow::Result<()> {
//     // Make sure that the user exists
//     sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES ( ?1 )", user)
//         .execute(&mut *conn)
//         .await?;
//
//     // Update user to have twitch_user name
//     sqlx::query!(
//         "UPDATE users SET twitch_user = ?2 WHERE id = ?1",
//         user,
//         twitch_user
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }

pub async fn get_user(conn: &mut SqliteConnection, user_id: &UserID) -> Result<User> {
    todo!()
    // Ok(sqlx::query_as!(
    //     User,
    //     r#"
    //     SELECT id, twitch_id, github_user
    //     FROM users
    //     WHERE id = ?
    //     "#,
    //     user_id
    // )
    // .fetch_one(&mut *conn)
    // .await?)

    // Ok(User {
    //     id: r.id,
    //     twitch_user: r.twitch_user,
    //     github_user: r.github_user,
    // })
}

pub async fn get_user_from_twitch_user(
    conn: &mut SqliteConnection,
    twitch_user_id: &str,
) -> Result<UserID> {
    let id = sqlx::query!(
        r#"SELECT id
           FROM users
           WHERE twitch_id = ?"#,
        twitch_user_id
    )
    .fetch_optional(&mut *conn)
    .await?;

    match id {
        Some(record) => Ok(record.id),
        None => {
            let new_id = sqlx::query!(
                r#"INSERT INTO users (twitch_id) VALUES (?)"#,
                twitch_user_id
            )
            .execute(&mut *conn)
            .await?
            .last_insert_rowid();

            Ok(new_id)
        }
    }
}

pub async fn get_message_count_from_today(
    conn: &mut SqliteConnection,
    user_id: &UserID,
) -> Result<i32> {
    Ok(sqlx::query!(
        r#"
        SELECT
          count(*) as c
        FROM
          TWITCH_CHAT_HISTORY
        WHERE
          user_id = ?
          AND TIMESTAMP >= '2022-05-13'
        "#,
        user_id
    )
    .fetch_one(&mut *conn)
    .await?
    .c)
}

pub async fn create_twitch_user_CHAT(
    conn: &mut SqliteConnection,
    twitch_user_id: &str,
    twitch_user_login: &str,
) -> Result<()> {
    sqlx::query!("INSERT OR IGNORE INTO twitch_users (id, login, display_name, broadcaster_type, account_type )
                 VALUES                              (?1, ?2,    ?3,           ?4              , ?5 )", 
                 twitch_user_id, twitch_user_login, twitch_user_login, "", "")
        .execute(&mut *conn)
        .await.unwrap();
    Ok(())
}

pub async fn save_twitch_message(
    conn: &mut SqliteConnection,
    twitch_user_id: &str,
    message: &str,
) -> Result<()> {
    let user_id = get_user_from_twitch_user(conn, twitch_user_id).await?;

    sqlx::query!(
        r#"INSERT INTO TWITCH_CHAT_HISTORY (user_id, msg)
           VALUES ( ?1, ?2 )"#,
        user_id,
        message
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub type TwitchUserID = i64;
pub struct TwitchUser {
    id: TwitchUserID,
    login: String,
    display_name: String,
    broadcaster_type: String,
    account_type: String,
    offline_image_url: Option<String>,
    profile_image_url: Option<String>,
    account_created_at: Option<String>,
}

async fn create_twitch_user(conn: &mut SqliteConnection, twitch_user: TwitchUser) -> Result<()> {
    sqlx::query!(
        "INSERT INTO twitch_users (id, login, display_name, broadcaster_type, account_type)
            VALUES ( ?1, ?2, ?3, '', '' )",
        twitch_user.id,
        twitch_user.login,
        twitch_user.display_name
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn get_twitch_user(conn: &mut SqliteConnection, id: TwitchUserID) -> Result<TwitchUser> {
    Ok(
        sqlx::query_as!(TwitchUser, "SELECT * FROM twitch_users WHERE id = ?", id)
            .fetch_one(&mut *conn)
            .await?,
    )
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use super::*;

    async fn get_test_database() -> anyhow::Result<SqlitePool> {
        let pool = SqlitePool::connect(":memory:").await?;
        sqlx::migrate!().run(&pool).await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_database_connects() -> anyhow::Result<()> {
        get_test_database().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_user() -> anyhow::Result<()> {
        let pool = get_test_database().await?;
        let mut conn = pool.acquire().await?;

        create_twitch_user(
            &mut conn,
            TwitchUser {
                id: 1234,
                login: "test_user".to_string(),
                display_name: "Test User".to_string(),
                broadcaster_type: "".to_string(),
                account_type: "".to_string(),
                offline_image_url: None,
                profile_image_url: None,
                account_created_at: None,
            },
        )
        .await?;

        let user = get_twitch_user(&mut conn, 1234).await?;
        assert_eq!(user.id, 1234);

        Ok(())
    }
}
