#![allow(dead_code, unused_imports)]

// use subd_macros::UpdateAttr;
use subd_macros::database_model;

// use anyhow::Result;

type UserID = i32;
type TwitchID = String;
type GithubID = String;

pub trait DatabaseModel {}

#[database_model]
mod user_model {
    use super::*;

    pub struct Model {
        #[immutable]
        id: UserID,

        twitch_id: Option<TwitchID>,
        github_id: Option<GithubID>,
    }

    impl Model {
        pub fn new(id: UserID, twitch_id: Option<TwitchID>, github_id: Option<GithubID>) -> Self {
            Self {
                id,
                twitch_id,
                github_id,
            }
        }

        pub fn read(_id: UserID) -> Self {
            Default::default()
        }

        pub fn save(self) -> Self {
            self
        }
    }
}

//
// #[derive(Default)]
// struct UpdateUser {
//     // id: UserID,
//     twitch_id: Option<Option<TwitchID>>,
//     github_id: Option<Option<GithubID>>,
// }
//
// impl User {
//     fn create(self) -> Self {
//         // do some db stuff and validation
//         self
//     }
//
//     fn save(self) -> Self {
//         // do some db stuff
//         self
//     }
//
//     fn update(mut self, update: UpdateUser) -> Self {
//         if let Some(twitch_id) = update.twitch_id {
//             self.twitch_id = twitch_id
//         }
//
//         if let Some(github_id) = update.github_id {
//             self.github_id = github_id
//         }
//
//         self.save()
//     }
//
//     fn delete(id: UserID) -> () {
//         //
//     }
//
//     fn get_by_id(id: UserID) -> Result<Option<Self>> {
//         // run some db stuff
//         Ok(Some(Default::default()))
//     }
// }

//

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
    let user = user_model::Model::new(1, Some("twitch-1234".to_string()), None);
    println!("original user: {:?}", user);

    // found out we had github user
    let new_user = user.update(user_model::ModelUpdate {
        github_id: Some(Some("github-foo".to_string())),
        ..Default::default()
    });

    println!("updated_user: {:#?}", new_user);
    Ok(())
}
