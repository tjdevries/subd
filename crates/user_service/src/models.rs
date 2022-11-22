// models
use anyhow::Result;

use sqlx::PgConnection;
use subd_macros::database_model;
use subd_types::UserPlatform;
use subd_types::UserRoles;

#[database_model]
pub mod user_messages {
    use super::*;

    pub struct Model {
        pub user_id: sqlx::types::Uuid,
        pub platform: subd_types::UserPlatform,
        pub contents: String,
    }
}

impl user_messages::Model {
    pub async fn save(self, conn: &mut PgConnection) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO user_messages (user_id, platform, contents)
            VALUES ($1, $2, $3)
            RETURNING user_id, platform as "platform: UserPlatform", contents
            "#,
            self.user_id,
            self.platform as _,
            self.contents
        )
        .fetch_one(conn)
        .await?)
    }
}

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
        #[primary_key]
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
    pub async fn save(self, conn: &mut PgConnection) -> Result<Self> {
        // Ok(sqlx::query_as!(
        //     Self,
        //     r#"
        //     INSERT INTO user_roles (id, twitch_id, github_id)
        //       VALUES ($1, $2, $3) ON CONFLICT (id) DO
        //       UPDATE
        //       SET twitch_id=$2, github_id=$3
        //       RETURNING *
        //     "#,
        //     self.id,
        //     self.twitch_id,
        //     self.github_id
        // )
        // .execute(&mut *conn)
        // .await?)

        // This is for update
        // let query = "
        //     UPDATE user_roles
        //         SET field1=$2, field2=$3
        //         WHERE primary_key=$1
        //         RETURNING *
        // ";
        // sqlx::query(query).execute(conn).await?;

        todo!()
    }
}
