#![allow(dead_code)]

use std::collections::HashSet;

use anyhow::Result;
use sqlx::SqliteConnection;
use subd_types::{Role, TwitchSubLevel, UserID, UserRoles};
use tracing::info;
use twitch_irc::message::PrivmsgMessage;

// Query for finding if anyone with this tag has messaged today
// SELECT count(*) as c
//   WHERE exists(
//     SELECT *
//     FROM TWITCH_CHAT_HISTORY
//       INNER JOIN TWITCH_USERS on TWITCH_USERS.id = USERS.twitch_id
//       INNER JOIN USERS on TWITCH_CHAT_HISTORY.user_id = USERS.id
//       INNER JOIN USER_ROLES on TWITCH_CHAT_HISTORY.user_id = USER_ROLES.user_id
//     WHERE
//       USER_ROLES.is_twitch_mod = true
//   )

pub async fn update_user_roles_once_per_day(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<UserRoles> {
    let user_roles = subd_db::get_user_roles(conn, user_id).await?;
    let twitch_roles = get_twitch_roles_from_msg(msg);

    // if user_roles.is_twitch_mod() == twitch_roles.is_twitch_mod()
    //     && user_roles.is_twitch_vip() == twitch_roles.is_twitch_vip()
    //     && user_roles.is_twitch_founder() == twitch_roles.is_twitch_founder()
    //     && user_roles.is_twitch_sub() == twitch_roles.is_twitch_sub()
    // {
    //     let record = sqlx::query_as!(
    //         UserRoles,
    //         "
    //         select
    //             is_github_sponsor,
    //             is_twitch_mod,
    //             is_twitch_vip,
    //             is_twitch_founder,
    //             is_twitch_sub,
    //             is_twitch_staff
    //         FROM USER_ROLES
    //         WHERE
    //           user_id = ?1 AND date(verified_date) = date(CURRENT_TIMESTAMP)
    //           ORDER BY verified_date DESC
    //         ",
    //         user_id
    //     )
    //     .fetch_optional(&mut *conn)
    //     .await?;
    //
    //     // If we have any record, that means we've updated already today,
    //     // so don't do that again
    //     if let Some(record) = record {
    //         return Ok(record);
    //     }
    // }

    Ok(update_user_roles(conn, user_id, msg).await?)
}

fn get_twitch_roles_from_msg(msg: &PrivmsgMessage) -> UserRoles {
    let mut roles = HashSet::new();
    if msg
        .badges
        .iter()
        .any(|b| b.name == "moderator" || b.name == "broadcaster")
    {
        roles.insert(Role::TwitchMod);
    }

    if msg.badges.iter().any(|b| b.name == "vip") {
        roles.insert(Role::TwitchVIP);
    }
    if msg.badges.iter().any(|b| b.name == "founder") {
        roles.insert(Role::TwitchFounder);
    }
    if msg.badges.iter().any(|b| b.name == "subscriber") {
        roles.insert(Role::TwitchSub(TwitchSubLevel::Unknown));
    }
    if msg.badges.iter().any(|b| b.name == "staff") {
        roles.insert(Role::TwitchStaff);
    }

    UserRoles {
        roles,
        ..Default::default()
    }
}

async fn get_user_role_from_user_id_and_msg(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<UserRoles> {
    let is_github_sponsor = match subd_db::get_github_info_for_user(&mut *conn, user_id).await? {
        Some(github_user) => subd_gh::is_user_sponsoring(&github_user.login).await?,
        None => false,
    };

    let mut twitch_roles = get_twitch_roles_from_msg(msg);
    if is_github_sponsor {
        twitch_roles.add_role(Role::GithubSponsor {
            tier: "UNKNOWN".to_string(),
        });
    }

    Ok(twitch_roles)
}

async fn update_user_roles(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<UserRoles> {
    let user_roles = get_user_role_from_user_id_and_msg(conn, user_id, msg).await?;

    info!(user_name = ?msg.sender.name, updated_roles = %user_roles, "updating user roles");
    subd_db::set_user_roles(conn, user_id, &user_roles).await?;

    Ok(user_roles)
}
