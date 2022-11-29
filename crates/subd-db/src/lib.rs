#![allow(dead_code)]

/// Get a connection to the database. Should not be called from a lot of places
/// (I think it should only be called from basically your "main" area... but we
/// will have to explore that a bit more).
pub async fn get_db_pool() -> sqlx::PgPool {
    sqlx::PgPool::connect(subd_types::consts::get_database_url().as_str())
        .await
        .expect("To get postgres pool connection")
}
// pub async fn set_github_info_for_user(
//     conn: &mut PgConnection,
//     user: &UserID,
//     github_login: &str,
// ) -> Result<()> {
//     let github_user = subd_gh::get_github_user(github_login).await?.ok_or(
//         anyhow::anyhow!("Could not find github user: {:?}", github_login),
//     )?;
//
//     sqlx::query!(
//         "
//         INSERT INTO github_users (id, login, name) VALUES (?1, ?2, ?3)
//             ON CONFLICT(id) DO UPDATE SET login=?2, name=?3
//         ",
//         github_user.id,
//         github_user.login,
//         github_user.name
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     // let result = sqlx::query!("SELECT id from github_users WHERE id = ?1", github_user.id)
//     //     .fetch_optional(&mut *conn)
//     //     .await?;
//     // if result.is_some() {
//     //     sqlx::query!(
//     //         "UPDATE github_users set (id, login, name) = (?1, ?2, ?3) WHERE id = ?1",
//     //         github_user.id,
//     //         github_user.login,
//     //         github_user.name
//     //     )
//     //     .execute(&mut *conn)
//     //     .await?;
//     // } else {
//     //     // Add github_user info to github_users
//     //     // TODO: Probably should delete any existing github user? sqlx::query!("DELETE
//     //     sqlx::query!(
//     //         "INSERT INTO github_users (id, login, name) VALUES (?1, ?2, ?3)",
//     //         github_user.id,
//     //         github_user.login,
//     //         github_user.name
//     //     )
//     //     .execute(&mut *conn)
//     //     .await?;
//     // }
//
//     // Update user to have github_user name
//     sqlx::query!(
//         "UPDATE users SET github_id = ?2 WHERE id = ?1",
//         user,
//         github_user.id
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn get_github_info_for_user(
//     conn: &mut PgConnection,
//     user_id: &UserID,
// ) -> Result<Option<GithubUser>> {
//     // TODO: Rewrite this as one query
//     let record =
//         sqlx::query!("SELECT github_id FROM USERS WHERE id = ?1", user_id)
//             .fetch_one(&mut *conn)
//             .await?;
//
//     let github_id = match record.github_id {
//         Some(github_id) => github_id,
//         None => return Ok(None),
//     };
//
//     let record = sqlx::query!(
//         "SELECT id, login, name FROM github_users WHERE id = ?1",
//         github_id
//     )
//     .fetch_one(&mut *conn)
//     .await?;
//
//     Ok(Some(GithubUser {
//         id: record.id,
//         login: record.login,
//         name: record.name,
//     }))
// }
//
// // async fn set_twitch_user_for_user(
// //     conn: &mut PgConnection,
// //     user: &UserID,
// //     twitch_user: &str,
// // ) -> anyhow::Result<()> {
// //     // Make sure that the user exists
// //     sqlx::query!("INSERT OR IGNORE INTO users (id) VALUES ( ?1 )", user)
// //         .execute(&mut *conn)
// //         .await?;
// //
// //     // Update user to have twitch_user name
// //     sqlx::query!(
// //         "UPDATE users SET twitch_user = ?2 WHERE id = ?1",
// //         user,
// //         twitch_user
// //     )
// //     .execute(&mut *conn)
// //     .await?;
// //
// //     Ok(())
// // }
//
// pub async fn get_user(
//     _conn: &mut PgConnection,
//     _user_id: &UserID,
// ) -> Result<User> {
//     todo!()
//     // Ok(sqlx::query_as!(
//     //     User,
//     //     r#"
//     //     SELECT id, twitch_id, github_user
//     //     FROM users
//     //     WHERE id = ?
//     //     "#,
//     //     user_id
//     // )
//     // .fetch_one(&mut *conn)
//     // .await?)
//
//     // Ok(User {
//     //     id: r.id,
//     //     twitch_user: r.twitch_user,
//     //     github_user: r.github_user,
//     // })
// }
//
// pub async fn get_user_from_twitch_user(
//     conn: &mut PgConnection,
//     twitch_user_id: &str,
// ) -> Result<UserID> {
//     let id = sqlx::query!(
//         r#"SELECT id
//            FROM users
//            WHERE twitch_id = ?"#,
//         twitch_user_id
//     )
//     .fetch_optional(&mut *conn)
//     .await?;
//
//     match id {
//         Some(record) => Ok(record.id),
//         None => {
//             let new_id = sqlx::query!(
//                 r#"INSERT INTO users (twitch_id) VALUES (?)"#,
//                 twitch_user_id
//             )
//             .execute(&mut *conn)
//             .await?
//             .last_insert_rowid();
//
//             Ok(new_id)
//         }
//     }
// }
//
// pub async fn get_user_from_twitch_user_name(
//     conn: &mut PgConnection,
//     display_name: &str,
// ) -> Result<Option<UserID>> {
//     let display_name = display_name.to_lowercase();
//
//     let record = sqlx::query!(
//         r#"
//         SELECT users.id
//             FROM users
//                 JOIN twitch_users ON twitch_users.id = users.twitch_id
//             WHERE twitch_users.display_name = ?1;
//           "#,
//         display_name
//     )
//     .fetch_optional(&mut *conn)
//     .await?;
//
//     match record {
//         Some(record) => Ok(Some(record.id)),
//         None => Ok(None),
//     }
// }
//
// pub async fn get_message_count_from_today(
//     conn: &mut PgConnection,
//     user_id: &UserID,
// ) -> Result<i32> {
//     Ok(sqlx::query!(
//         r#"
//         SELECT
//           count(*) as c
//         FROM
//           TWITCH_CHAT_HISTORY
//         WHERE
//           user_id = ?
//           AND TIMESTAMP >= '2022-05-13'
//         "#,
//         user_id
//     )
//     .fetch_one(&mut *conn)
//     .await?
//     .c)
// }
//
//
// pub async fn save_twitch_message(
//     conn: &mut PgConnection,
//     twitch_user_id: &str,
//     message: &str,
// ) -> Result<()> {
//     let user_id = get_user_from_twitch_user(conn, twitch_user_id).await?;
//
//     sqlx::query!(
//         r#"INSERT INTO TWITCH_CHAT_HISTORY (user_id, msg)
//            VALUES ( ?1, ?2 )"#,
//         user_id,
//         message
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }
//
// pub type TwitchUserID = i64;
// pub struct TwitchUser {
//     pub id: TwitchUserID,
//     pub login: String,
//     pub display_name: String,
//     pub broadcaster_type: String,
//     pub account_type: String,
//     pub offline_image_url: Option<String>,
//     pub profile_image_url: Option<String>,
//     pub account_created_at: Option<String>,
// }
//
// async fn create_twitch_user(
//     conn: &mut PgConnection,
//     twitch_user: TwitchUser,
// ) -> Result<()> {
//     sqlx::query!(
//         "INSERT INTO twitch_users (id, login, display_name, broadcaster_type, account_type)
//             VALUES ( ?1, ?2, ?3, '', '' )",
//         twitch_user.id,
//         twitch_user.login,
//         twitch_user.display_name
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn get_twitch_user_from_user_id(
//     conn: &mut PgConnection,
//     id: UserID,
// ) -> Result<TwitchUser> {
//     Ok(sqlx::query_as!(
//         TwitchUser,
//         "SELECT twitch_users.*
//             FROM twitch_users
//                 INNER JOIN users on twitch_users.id = users.twitch_id
//             WHERE users.id = ?
//         ",
//         id
//     )
//     .fetch_one(&mut *conn)
//     .await?)
// }
//
// pub async fn set_user_roles(
//     conn: &mut PgConnection,
//     user_id: &UserID,
//     roles: &UserRoles,
// ) -> Result<()> {
//     let is_github_sponsor = roles.is_github_sponsor();
//     let is_twitch_mod = roles.is_twitch_mod();
//     let is_twitch_vip = roles.is_twitch_vip();
//     let is_twitch_founder = roles.is_twitch_founder();
//     let is_twitch_sub = roles.is_twitch_sub();
//     let is_twitch_staff = roles.is_twitch_staff();
//
//     sqlx::query!(
//         "INSERT INTO user_roles (
//             user_id,
//             is_github_sponsor,
//             is_twitch_mod,
//             is_twitch_vip,
//             is_twitch_founder,
//             is_twitch_sub,
//             is_twitch_staff
//         ) VALUES (
//             ?1,
//             ?2,
//             ?3,
//             ?4,
//             ?5,
//             ?6,
//             ?7
//         )",
//         user_id,
//         is_github_sponsor,
//         is_twitch_mod,
//         is_twitch_vip,
//         is_twitch_founder,
//         is_twitch_sub,
//         is_twitch_staff,
//     )
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }
//
// pub async fn get_user_roles(
//     conn: &mut PgConnection,
//     user_id: &UserID,
// ) -> Result<UserRoles> {
//     struct UserRoleRow {
//         is_github_sponsor: bool,
//         is_twitch_mod: bool,
//         is_twitch_vip: bool,
//         is_twitch_founder: bool,
//         is_twitch_sub: bool,
//         is_twitch_staff: bool,
//     }
//
//     Ok(
//         match sqlx::query_as!(
//             UserRoleRow,
//             "
//     SELECT
//         is_github_sponsor,
//         is_twitch_mod,
//         is_twitch_vip,
//         is_twitch_founder,
//         is_twitch_sub,
//         is_twitch_staff
//     FROM user_roles
//         WHERE user_id = ?1
//     ORDER BY verified_date DESC
//     LIMIT 1
//             ",
//             user_id
//         )
//         .fetch_optional(&mut *conn)
//         .await?
//         {
//             Some(row) => UserRoles {
//                 roles: {
//                     let mut h = HashSet::new();
//                     if row.is_github_sponsor {
//                         h.insert(Role::GithubSponsor {
//                             tier: "UNKNOWN".to_string(),
//                         });
//                     }
//
//                     if row.is_twitch_mod {
//                         h.insert(Role::TwitchMod);
//                     }
//                     if row.is_twitch_vip {
//                         h.insert(Role::TwitchVIP);
//                     }
//                     if row.is_twitch_founder {
//                         h.insert(Role::TwitchFounder);
//                     }
//                     if row.is_twitch_sub {
//                         h.insert(Role::TwitchSub(TwitchSubLevel::Tier1));
//                     }
//                     if row.is_twitch_staff {
//                         h.insert(Role::TwitchStaff);
//                     }
//
//                     h
//                 },
//             },
//             None => UserRoles::default(),
//         },
//     )
// }
//
// #[cfg(test)]
// mod tests {
//     use sqlx::PgPool;
//
//     use super::*;
//
//     async fn get_test_database() -> anyhow::Result<PgPool> {
//         let pool = PgPool::connect(":memory:").await?;
//         sqlx::migrate!().run(&pool).await?;
//
//         Ok(pool)
//     }
//
//     #[tokio::test]
//     async fn test_database_connects() -> anyhow::Result<()> {
//         get_test_database().await?;
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_insert_user() -> anyhow::Result<()> {
//         let pool = get_test_database().await?;
//         let mut conn = pool.acquire().await?;
//
//         create_twitch_user(
//             &mut conn,
//             TwitchUser {
//                 id: 1234,
//                 login: "test_user".to_string(),
//                 display_name: "Test User".to_string(),
//                 broadcaster_type: "".to_string(),
//                 account_type: "".to_string(),
//                 offline_image_url: None,
//                 profile_image_url: None,
//                 account_created_at: None,
//             },
//         )
//         .await?;
//
//         // let user = get_twitch_user(&mut conn, 1234).await?;
//         // assert_eq!(user.id, 1234);
//
//         Ok(())
//     }
// }
