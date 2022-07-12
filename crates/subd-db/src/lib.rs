#![allow(dead_code)]

use anyhow::Result;
use sqlx::{Connection, SqliteConnection};
use subd_types::{GithubUser, UserID, UserRoles};

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
    SqliteConnection::connect("sqlite:/home/tjdevries/git/subd/subd.db")
        .await
        .expect("To connect to the database")
}

pub async fn set_github_info_for_user(
    conn: &mut SqliteConnection,
    user: &UserID,
    github_login: &str,
) -> Result<()> {
    let github_user = subd_gh::get_github_user(github_login)
        .await?
        .ok_or(anyhow::anyhow!(
            "Could not find github user: {:?}",
            github_login
        ))?;

    sqlx::query!(
        "
        INSERT INTO github_users (id, login, name) VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET login=?2, name=?3
        ",
        github_user.id,
        github_user.login,
        github_user.name
    )
    .execute(&mut *conn)
    .await?;

    // let result = sqlx::query!("SELECT id from github_users WHERE id = ?1", github_user.id)
    //     .fetch_optional(&mut *conn)
    //     .await?;
    // if result.is_some() {
    //     sqlx::query!(
    //         "UPDATE github_users set (id, login, name) = (?1, ?2, ?3) WHERE id = ?1",
    //         github_user.id,
    //         github_user.login,
    //         github_user.name
    //     )
    //     .execute(&mut *conn)
    //     .await?;
    // } else {
    //     // Add github_user info to github_users
    //     // TODO: Probably should delete any existing github user? sqlx::query!("DELETE
    //     sqlx::query!(
    //         "INSERT INTO github_users (id, login, name) VALUES (?1, ?2, ?3)",
    //         github_user.id,
    //         github_user.login,
    //         github_user.name
    //     )
    //     .execute(&mut *conn)
    //     .await?;
    // }

    // Update user to have github_user name
    sqlx::query!(
        "UPDATE users SET github_id = ?2 WHERE id = ?1",
        user,
        github_user.id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn get_github_info_for_user(
    conn: &mut SqliteConnection,
    user_id: &UserID,
) -> Result<Option<GithubUser>> {
    // TODO: Rewrite this as one query
    let record = sqlx::query!("SELECT github_id FROM USERS WHERE id = ?1", user_id)
        .fetch_one(&mut *conn)
        .await?;

    let github_id = match record.github_id {
        Some(github_id) => github_id,
        None => return Ok(None),
    };

    let record = sqlx::query!(
        "SELECT id, login, name FROM github_users WHERE id = ?1",
        github_id
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(Some(GithubUser {
        id: record.id,
        login: record.login,
        name: record.name,
    }))
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

pub async fn get_user(_conn: &mut SqliteConnection, _user_id: &UserID) -> Result<User> {
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

pub async fn get_user_from_twitch_user_name(
    conn: &mut SqliteConnection,
    display_name: &str,
) -> Result<Option<UserID>> {
    let display_name = display_name.to_lowercase();

    let record = sqlx::query!(
        r#"
        SELECT users.id
            FROM users
                JOIN twitch_users ON twitch_users.id = users.twitch_id
            WHERE twitch_users.display_name = ?1;
          "#,
        display_name
    )
    .fetch_optional(&mut *conn)
    .await?;

    match record {
        Some(record) => Ok(Some(record.id)),
        None => Ok(None),
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

pub async fn create_twitch_user_chat(
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

pub async fn set_user_roles(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    roles: UserRoles,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO user_roles (
            user_id, 
            is_github_sponsor,
            is_twitch_mod,
            is_twitch_vip,
            is_twitch_founder,
            is_twitch_sub,
            is_twitch_staff
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6,
            ?7
        )",
        user_id,
        roles.is_github_sponsor,
        roles.is_twitch_mod,
        roles.is_twitch_vip,
        roles.is_twitch_founder,
        roles.is_twitch_sub,
        roles.is_twitch_staff,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn get_user_roles(conn: &mut SqliteConnection, user_id: &UserID) -> Result<UserRoles> {
    Ok(sqlx::query_as!(
        UserRoles,
        "
SELECT 
    is_github_sponsor,
    is_twitch_mod,
    is_twitch_vip,
    is_twitch_founder,
    is_twitch_sub,
    is_twitch_staff
FROM user_roles
    WHERE user_id = ?1
ORDER BY verified_date DESC
LIMIT 1
        ",
        user_id
    )
    .fetch_optional(&mut *conn)
    .await?
    .unwrap_or_default())
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
