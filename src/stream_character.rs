use crate::move_transition;
use crate::move_transition_bootstrap;
use crate::obs;
use anyhow::Result;
use obws::Client as OBSClient;
use sqlx::PgPool;
use std::path::Path;
use subd_macros::database_model;

#[database_model]
pub mod user_stream_character_information {
    use super::*;

    pub struct Model {
        pub username: String,
        pub obs_character: String,
        pub voice: String,
        pub random: bool,
    }
}

impl user_stream_character_information::Model {
    #[allow(dead_code)]

    pub async fn save(self, pool: &PgPool) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
            INSERT INTO user_stream_character_information
            (username, obs_character, voice)
            VALUES ( $1, $2, $3 )
            ON CONFLICT (username)
            DO UPDATE SET
            obs_character = $2,
            voice = $3
            RETURNING username, obs_character, voice, random
        "#,
            self.username,
            self.obs_character,
            self.voice
        )
        .fetch_one(pool)
        .await?)
    }
}

pub async fn get_voice_from_username(
    pool: &PgPool,
    username: &str,
) -> Result<String> {
    let res = sqlx::query!(
        "SELECT voice FROM user_stream_character_information WHERE username = $1",
        username
    ).fetch_one(pool).await?;
    Ok(res.voice)
}

// =======================================================================
//
// Creating the Character in OBS w/ all the Filters
// This might not be the best place for this
pub async fn create_new_obs_character(
    base_source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene = obs::CHARACTERS_SCENE.to_string();

    // TODO: Make this relative and configurable for where the user store's it's OBS Assets
    let filename = format!(
        "/home/begin/stream/Stream/StreamCharacters/{}.png",
        base_source
    );

    // TODO: We need to pull in this source
    let image_source = obws::requests::custom::source_settings::ImageSource {
        file: Path::new(&filename),
        ..Default::default()
    };

    // Figure out GIF VS Image
    let _ = obs_client
        .inputs()
        .create(obws::requests::inputs::Create {
            scene: &scene,
            input: &base_source,
            kind: "image_source",
            settings: Some(image_source),
            enabled: Some(true),
        })
        .await;

    // TODO: Fix this path
    // We need to include this in the assets of the project
    let speech_bubble = obws::requests::custom::source_settings::ImageSource {
        file: Path::new(
            "/home/begin/stream/Stream/StreamCharacters/speech_bubble.png",
        ),
        ..Default::default()
    };
    let speech_source_name = format!("{}-speech_bubble", base_source);
    let _ = obs_client
        .inputs()
        .create(obws::requests::inputs::Create {
            scene: &scene,
            input: &speech_source_name,
            kind: "image_source",
            settings: Some(speech_bubble),
            enabled: Some(true),
        })
        .await;

    // let font_flags = obws::common::FontFlags{ }
    let font = obws::requests::custom::source_settings::Font {
        face: "Arial",
        size: 256,
        style: "Regular",
        ..Default::default()
    };

    // TODO: THESE COLORS DON'T WORK!!!
    let color1 = rgb::RGBA::new(255, 0, 132, 0);
    let color2 = rgb::RGBA::new(0, 3, 255, 1);

    let text_settings =
        obws::requests::custom::source_settings::TextFt2SourceV2 {
            outline: true,
            drop_shadow: true,
            text: "This Rules we are doing something!",
            color1,
            color2,
            font,
            custom_width: 5,
            log_lines: 5,
            word_wrap: false,
            ..Default::default() // We might want to experiment from file
        };

    let text_source_name = format!("{}-text", base_source);
    let _ = obs_client
        .inputs()
        .create(obws::requests::inputs::Create {
            scene: &scene,
            input: &text_source_name,
            kind: "text_ft2_source_v2",
            settings: Some(text_settings),
            enabled: Some(true),
        })
        .await;

    // ======================================================

    // This is creating the Text Transform Filter
    // Create Move-Value for 3D Transform Filter
    let filter_name = format!("Transform{}-text", base_source);
    let move_text_filter = move_transition::MoveTextFilter {
        setting_name: "text".to_string(),
        setting_text: "Ok NOW".to_string(),
        value_type: 4,
        ..Default::default()
    };
    let new_filter = obws::requests::filters::Create {
        source: &text_source_name,
        filter: &filter_name,
        kind: "move_value_filter",
        settings: Some(move_text_filter),
    };
    if let Err(err) = obs_client.filters().create(new_filter).await {
        println!("Error Creating Filter: {filter_name} | {:?}", err);
    };

    let file_path = "./obs_data/move_transition_show_source.json";
    let filter_name = format!("Show{}", base_source);
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &base_source,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;

    let filter_name = format!("Hide{}", base_source);
    let file_path = "./obs_data/move_transition_hide_source.json";
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &base_source,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;

    let filter_name = format!("Show{}-text", base_source);
    let file_path = "./obs_data/move_transition_show_text.json";
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &text_source_name,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;

    let filter_name = format!("Hide{}-text", base_source);
    let file_path = "./obs_data/move_transition_hide_text.json";
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &text_source_name,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;

    let filter_name = format!("Show{}-speech_bubble", base_source);
    let file_path = "./obs_data/move_transition_show_speech_bubble.json";
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &speech_source_name,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;

    let filter_name = format!("Hide{}-speech_bubble", base_source);
    let file_path = "./obs_data/move_transition_hide_speech_bubble.json";
    let _ = move_transition_bootstrap::create_move_source_filter_from_file(
        &scene,
        &speech_source_name,
        &filter_name,
        file_path,
        &obs_client,
    )
    .await;
    Ok(())
}
