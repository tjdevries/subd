// models
use anyhow::Result;

use sqlx::PgConnection;
use subd_macros::database_model;
use subd_types::UserID;
use subd_types::UserRoles;

#[database_model]
pub mod user_roles {
    use super::*;

    pub fn to_user_roles(m: Model) -> UserRoles {
        // Map to roles
        let mut roles = UserRoles {
            roles: std::collections::HashSet::new(),
        };

        if m.is_github_sponsor {
            roles.add_role(subd_types::Role::GithubSponsor {
                tier: "unknown".to_string(),
            });
        }

        if m.is_twitch_mod {
            roles.add_role(subd_types::Role::TwitchMod);
        }

        if m.is_twitch_vip {
            roles.add_role(subd_types::Role::TwitchVIP);
        }

        if m.is_twitch_staff {
            roles.add_role(subd_types::Role::TwitchStaff);
        }

        if m.is_twitch_founder {
            roles.add_role(subd_types::Role::TwitchFounder);
        }

        // TODO: Handle subs correctly
        if m.is_twitch_sub {
            roles.add_role(subd_types::Role::TwitchSub(
                subd_types::TwitchSubLevel::Unknown,
            ));
        }

        roles
    }

    pub struct Model {
        #[immutable]
        pub user_id: sqlx::types::Uuid,

        // Github
        pub is_github_sponsor: bool,

        // Twitch
        pub is_twitch_mod: bool,
        pub is_twitch_vip: bool,
        pub is_twitch_founder: bool,
        pub is_twitch_sub: bool,
        pub is_twitch_staff: bool,
    }
}

impl user_roles::Model {
    // TODO: Would be awesome to just generate this... :)
    //       ITS NOT AN ORM CHAT I SWEAR
    pub async fn read(
        conn: &mut PgConnection,
        id: &UserID,
    ) -> Result<Option<Self>> {
        let x = sqlx::query_as!(
            user_roles::Model,
            r#"
            SELECT 
                user_id,
                is_github_sponsor,
                is_twitch_mod,
                is_twitch_vip,
                is_twitch_founder,
                is_twitch_sub,
                is_twitch_staff
            FROM user_roles where user_id = $1
            "#,
            id.0
        )
        .fetch_optional(conn)
        .await?;

        Ok(x)
    }

    pub async fn save(&self, conn: &mut PgConnection) -> Result<()> {
        todo!("{:?}", conn)
        // sqlx::query!(
        //     r#"
        //     INSERT INTO users (id, twitch_id, github_id)
        //       VALUES (?1, ?2, ?3) ON CONFLICT (id) DO
        //       UPDATE
        //       SET twitch_id=?2, github_id=?3
        //     "#,
        //     self.id,
        //     self.twitch_id,
        //     self.github_id
        // )
        // .execute(&mut *conn)
        // .await?;
        //
        // Ok(())
    }
}
