use subd_macros::database_model;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;

// CREATE TABLE skybox_requests (
//   blockade_id INT NOT NULL,
//   prompt TEXT NOT NULL,
//   skybox_style_id INT NOT NULL,
//   file_url TEXT, 
//   
//   created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
//   completed_at  TIMESTAMPTZ
// );


#[database_model]
pub mod skybox_requests {
    use super::*;

    pub struct Model {
        pub blockade_id: i32,
        pub prompt: String,
        pub skybox_style_id: i32,
        pub file_url: String,

        // TODO: Do I want to depend on Postgresql FKs for users?
        pub username: String,
        pub created_at: Option<OffsetDateTime>,
        pub completed_at: Option<OffsetDateTime>,
    }
}

