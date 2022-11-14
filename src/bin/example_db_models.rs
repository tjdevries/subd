#![allow(dead_code, unused_imports)]

// Some questions for next time:
// - how do we do joins? (and prevent endless querires)
// - how do we do lists of things?
//      - get all messages for user (sqlx)
//      - impl some sqlx stuff for these structs?
// - create, read, update, delete
// - think about: joins, many2many, many2one, one2many, constraints, indexes

use sqlx::{Connection, SqliteConnection};
// use subd_macros::UpdateAttr;
use subd_macros::database_model;

use anyhow::Result;
use async_trait::async_trait;

type UserID = i64;
type TwitchID = String;
type GithubID = String;

// TODO: I don't think I like these traits, they don't seem to give me anything
// on the other end. I'd rather just call methods directly on the models.
#[async_trait]
pub trait DatabaseModel {
    async fn save(conn: &mut impl Connection) -> Result<()>;
}

#[async_trait]
pub trait DatabaseModelWithKey: DatabaseModel + Sized {
    type Key;
    async fn read(
        conn: &mut SqliteConnection,
        id: Self::Key,
    ) -> Result<Option<Self>>;
}

#[database_model]
mod user_model {
    use super::*;

    pub struct Model {
        #[immutable]
        pub(super) id: UserID,

        pub(super) twitch_id: Option<TwitchID>,
        pub(super) github_id: Option<GithubID>,
    }
}

impl user_model::Model {
    pub async fn read(
        conn: &mut SqliteConnection,
        id: UserID,
    ) -> Result<Option<Self>> {
        let x = sqlx::query!(
            r#"
            SELECT id, twitch_id, github_id
              FROM users
              WHERE id = ?1
            "#,
            id
        )
        .fetch_optional(conn)
        .await?;

        Ok(x.map(|x| Self::new(x.id, x.twitch_id, x.github_id)))
    }

    pub async fn save(&self, conn: &mut SqliteConnection) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, twitch_id, github_id)
              VALUES (?1, ?2, ?3) ON CONFLICT (id) DO
              UPDATE
              SET twitch_id=?2, github_id=?3
            "#,
            self.id,
            self.twitch_id,
            self.github_id
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}

// Model:
//  id
//  twitch_id
//  github_id
//
// Update that model:
//  some of the fields (not all of them)
//
//  UpdateModel { twitch_id: Some(new_id), ..None }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut conn = subd_db::get_handle().await;

    // Create or retreivew a user
    let user = user_model::Model::new(1, Some("twitch-1234".to_string()), None);

    // Update user
    let new_user_result = user
        .update(
            &mut conn,
            user_model::ModelUpdate {
                github_id: Some(Some("github-foo".to_string())),
                ..Default::default()
            },
        )
        .await;

    let new_user = new_user_result.unwrap();

    println!("updated_user: {:#?}", new_user);
    Ok(())
}
