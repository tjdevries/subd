use anyhow::Result;
use sqlx::SqliteConnection;
use subd_types::{UserID, UserRoles};
use twitch_irc::message::PrivmsgMessage;

pub async fn update_user_roles_once_per_day(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<()> {
    let user_roles = subd_db::get_user_roles(conn, user_id).await?;
    let twitch_roles = get_twitch_roles_from_msg(msg);

    if user_roles.is_twitch_mod == twitch_roles.is_twitch_mod
        && user_roles.is_twitch_vip == twitch_roles.is_twitch_vip
        && user_roles.is_twitch_founder == twitch_roles.is_twitch_founder
        && user_roles.is_twitch_sub == twitch_roles.is_twitch_sub
    {
        let record = sqlx::query!(
        "select user_id from USER_ROLES where user_id = ?1 AND date(verified_date) = date(CURRENT_TIMESTAMP)",
        user_id).fetch_optional(&mut *conn).await?;

        // If we have any record, that means we've updated already today,
        // so don't do that again
        if record.is_some() {
            return Ok(());
        }
    }

    update_user_roles(conn, user_id, msg).await?;

    Ok(())
}

fn get_twitch_roles_from_msg(msg: &PrivmsgMessage) -> UserRoles {
    let is_twitch_mod = msg.badges.iter().any(|b| b.name == "moderator");
    let is_twitch_vip = msg.badges.iter().any(|b| b.name == "vip");
    let is_twitch_founder = msg.badges.iter().any(|b| b.name == "founder");
    let is_twitch_sub = msg.badges.iter().any(|b| b.name == "subscriber");

    UserRoles {
        is_twitch_mod,
        is_twitch_vip,
        is_twitch_founder,
        is_twitch_sub,
        is_github_sponsor: false,
    }
}

async fn get_user_role_from_user_id_and_msg(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<UserRoles> {
    let github_user = subd_db::get_github_info_for_user(&mut *conn, user_id).await?;
    let is_github_sponsor = if let Some(github_user) = github_user {
        subd_gh::is_user_sponsoring(&github_user.login).await?
    } else {
        false
    };

    let twitch_roles = get_twitch_roles_from_msg(msg);
    Ok(UserRoles {
        is_github_sponsor,
        ..twitch_roles
    })
}

async fn update_user_roles(
    conn: &mut SqliteConnection,
    user_id: &UserID,
    msg: &PrivmsgMessage,
) -> Result<()> {
    let user_roles = get_user_role_from_user_id_and_msg(conn, user_id, msg).await?;

    println!(
        "  Updating User Roles: {} -> {:?}",
        msg.sender.name, user_roles
    );
    subd_db::set_user_roles(conn, user_id, user_roles).await?;

    Ok(())
}
