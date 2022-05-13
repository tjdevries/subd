#![allow(dead_code)]

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
) -> anyhow::Result<()> {
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

async fn set_twitch_user_for_user(
    conn: &mut SqliteConnection,
    user: &UserID,
    twitch_user: &str,
) -> anyhow::Result<()> {
    // Make sure that the user exists
    sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES ( ?1 )", user)
        .execute(&mut *conn)
        .await?;

    // Update user to have twitch_user name
    sqlx::query!(
        "UPDATE users SET twitch_user = ?2 WHERE id = ?1",
        user,
        twitch_user
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn get_user(conn: &mut SqliteConnection, user_id: &UserID) -> anyhow::Result<User> {
    Ok(sqlx::query_as!(
        User,
        r#"
        SELECT id, twitch_user, github_user
        FROM users
        WHERE id = ?
        "#,
        user_id
    )
    .fetch_one(&mut *conn)
    .await?)

    // Ok(User {
    //     id: r.id,
    //     twitch_user: r.twitch_user,
    //     github_user: r.github_user,
    // })
}

pub async fn get_user_from_twitch_user(
    conn: &mut SqliteConnection,
    twitch_user: &str,
) -> anyhow::Result<UserID> {
    let id = sqlx::query!(r#"SELECT id FROM users WHERE twitch_user = ?"#, twitch_user)
        .fetch_optional(&mut *conn)
        .await?;

    match id {
        Some(record) => Ok(record.id),
        None => {
            let new_id = sqlx::query!(r#"INSERT INTO users (twitch_user) VALUES (?)"#, twitch_user)
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
) -> anyhow::Result<i32> {
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

pub async fn save_twitch_message(
    conn: &mut SqliteConnection,
    twitch_user: &str,
    message: &str,
) -> anyhow::Result<i64> {
    let user_id = get_user_from_twitch_user(conn, twitch_user).await?;

    sqlx::query!(
        r#"INSERT INTO TWITCH_CHAT_HISTORY (user_id, msg)
           VALUES ( ?1, ?2 )"#,
        user_id,
        message
    )
    .execute(&mut *conn)
    .await?;

    // Insert the task, then obtain the ID of this row
    //     let id = sqlx::query!(
    //         r#"
    // INSERT INTO NYX_LUL ( message )
    // VALUES ( ?1 )
    //         "#,
    //         message
    //     )
    //     .execute(&mut conn)
    //     .await?
    //     .last_insert_rowid();
    //
    //     Ok(id)
    Ok(1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
