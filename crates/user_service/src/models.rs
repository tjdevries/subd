// models
use anyhow::Result;

use sqlx::PgPool;
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
    #[allow(dead_code)]
    pub async fn save(self, pool: &PgPool) -> Result<Self> {
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
        .fetch_one(pool)
        .await?)
    }

    pub async fn get_messages_by_username(
        username: &str,
        pool: &PgPool,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
                        Self,
                        r#"
                        SELECT user_messages.user_id, platform as "platform: UserPlatform", contents
                        FROM user_messages
                        JOIN twitch_users ON user_messages.user_id = twitch_users.user_id
                        WHERE twitch_users.display_name = $1
                        AND NOT contents LIKE '!%'
                        AND NOT contents LIKE '@%'
                        AND array_length(regexp_split_to_array(contents, '\s+'), 1) >= 6
                        "#,
                        username
                    )
                    .fetch_all(pool)
                    .await?)
    }
}

#[database_model]
pub mod user_roles {
    use super::*;

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
}

#[allow(dead_code)]
impl user_roles::Model {
    pub fn empty(user_id: sqlx::types::Uuid) -> Self {
        Self {
            user_id,
            is_github_sponsor: false,
            is_twitch_mod: false,
            is_twitch_vip: false,
            is_twitch_founder: false,
            is_twitch_sub: false,
            is_twitch_staff: false,
        }
    }

    pub async fn save(self, conn: &PgPool) -> Result<Self> {
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

        Ok(sqlx::query_as!(
            Self,
            "INSERT INTO user_roles (
                user_id,
                is_github_sponsor,
                is_twitch_mod,
                is_twitch_vip,
                is_twitch_founder,
                is_twitch_sub,
                is_twitch_staff
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )  ON CONFLICT (user_id) DO UPDATE
            SET 
                is_github_sponsor = EXCLUDED.is_github_sponsor,
                is_twitch_mod = EXCLUDED.is_twitch_mod,
                is_twitch_vip = EXCLUDED.is_twitch_vip,
                is_twitch_founder = EXCLUDED.is_twitch_founder,
                is_twitch_sub = EXCLUDED.is_twitch_sub,
                is_twitch_staff = EXCLUDED.is_twitch_staff

             RETURNING 
                user_id,
                is_github_sponsor,
                is_twitch_mod,
                is_twitch_vip,
                is_twitch_founder,
                is_twitch_sub,
                is_twitch_staff
            ",
            self.user_id,
            self.is_github_sponsor,
            self.is_twitch_mod,
            self.is_twitch_vip,
            self.is_twitch_founder,
            self.is_twitch_sub,
            self.is_twitch_staff,
        )
        .fetch_one(conn)
        .await?)

        // This is for update
        // let query = "
        //     UPDATE user_roles
        //         SET field1=$2, field2=$3
        //         WHERE primary_key=$1
        //         RETURNING *
        // ";
        // sqlx::query(query).execute(conn).await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_user_messages_by_username() {
            let pool = subd_db::get_db_pool().await;

            let username = "beginbot";
            let messages =
                user_messages::Model::get_messages_by_username(username, &pool)
                    .await
                    .unwrap();

            assert!(
                !messages.is_empty(),
                "No messages found for the test user"
            );

            assert_eq!(messages.len(), 100);

            // we could populate a vector DB
            for message in messages {
                assert_eq!(message.platform, UserPlatform::Twitch);
                assert!(
                    !message.contents.is_empty(),
                    "Message content should not be empty"
                );
            }
        }
    }
}
