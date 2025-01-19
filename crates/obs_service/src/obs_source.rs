use anyhow::anyhow;
use anyhow::Result;
use num_traits::FromPrimitive;
use obws::requests::custom::source_settings::ColorRange;
use obws::requests::custom::source_settings::FfmpegSource;
use obws::requests::custom::source_settings::ImageSource;
use obws::requests::custom::source_settings::Slideshow;
use obws::requests::custom::source_settings::SlideshowFile;
use obws::requests::inputs::InputId;
use obws::requests::inputs::SetSettings;
use obws::requests::scene_items::{
    Position, Scale, SceneItemTransform, SetTransform,
};
use obws::requests::scenes::SceneId;
use obws::requests::sources::SaveScreenshot;
use obws::requests::sources::SourceId;
use obws::Client as OBSClient;
use sqlx::postgres::PgQueryResult;
use sqlx::types::BigDecimal;
use std::path::Path;

pub struct NewVoiceScene {
    pub voice: &'static str,
    pub music: &'static str,
    pub playlist_folder: Option<&'static str>,
}

pub const VOICE_TO_MUSIC: &[(&str, NewVoiceScene)] = &[
    // ("!yoga", NewVoiceScene{ voice: "Thomas", music: "Yoga-BG-Music", playlist_folder: Some("Yoga")}),
    (
        "!begin",
        NewVoiceScene {
            voice: "beginbot",
            music: "Carti-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!horror",
        NewVoiceScene {
            voice: "josh",
            music: "Horror",
            playlist_folder: Some("Horror"),
        },
    ),
    (
        "!comedy",
        NewVoiceScene {
            voice: "Fin",
            music: "Comedy",
            playlist_folder: Some("Comedy"),
        },
    ),
    (
        "!bond",
        NewVoiceScene {
            voice: "james",
            music: "Bond-BG-Music",
            playlist_folder: None,
        },
    ),
    // ("!streamer", NewVoiceScene{ voice: "melkey", music: "Lofi-HipHop-BG-Music", playlist_folder: Some("LofiHipHop")}),
    (
        "!streamer",
        NewVoiceScene {
            voice: "ethan",
            music: "Lofi-HipHop-BG-Music",
            playlist_folder: Some("LofiHipHop"),
        },
    ),
    (
        "!evil",
        NewVoiceScene {
            voice: "satan",
            music: "Evil-BG-Music",
            playlist_folder: Some("Evil"),
        },
    ),
    (
        "!good",
        NewVoiceScene {
            voice: "god",
            music: "Yoga-BG-Music",
            playlist_folder: Some("Yoga"),
        },
    ),
    (
        "!devito",
        NewVoiceScene {
            voice: "devito",
            music: "IASIP-BG-Music",
            playlist_folder: Some("IASIP"),
        },
    ),
    (
        "!yoga",
        NewVoiceScene {
            voice: "god",
            music: "Yoga-BG-Music",
            playlist_folder: Some("Yoga"),
        },
    ),
    (
        "!wes",
        NewVoiceScene {
            voice: "jeff",
            music: "Wes-BG-Music",
            playlist_folder: Some("Wes"),
        },
    ),
    (
        "!drama",
        NewVoiceScene {
            voice: "Ethan",
            music: "Dramatic-BG-Music",
            playlist_folder: Some("Drama"),
        },
    ),
    (
        "!greed",
        NewVoiceScene {
            voice: "Michael",
            music: "Greed-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!ken",
        NewVoiceScene {
            voice: "James",
            music: "KenBurns-BG-Music",
            playlist_folder: Some("KenBurns"),
        },
    ),
    // ("!hospital", NewVoiceScene{ voice: "prime", music: "Hospital-BG-Music", playlist_folder: Some("Hospital")}),
    (
        "!hospital",
        NewVoiceScene {
            voice: "Bella",
            music: "Hospital-BG-Music",
            playlist_folder: Some("Hospital"),
        },
    ),
    (
        "!sigma",
        NewVoiceScene {
            voice: "Ethan",
            music: "Sigma-BG-Music",
            playlist_folder: Some("Sigma"),
        },
    ),
    (
        "!news",
        NewVoiceScene {
            voice: "Ethan",
            music: "News-1-BG-Music",
            playlist_folder: Some("News"),
        },
    ),
    (
        "!sexy",
        NewVoiceScene {
            voice: "Charlotte",
            music: "Sexy-2-BG-Music",
            playlist_folder: Some("Sexy"),
        },
    ),
    (
        "!romcom",
        NewVoiceScene {
            voice: "Fin",
            music: "Romcom-BG-Music",
            playlist_folder: Some("RomCom"),
        },
    ),
    (
        "!chef",
        NewVoiceScene {
            voice: "Giovanni",
            music: "Chef-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!carti",
        NewVoiceScene {
            voice: "Random",
            music: "Carti-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!nerds",
        NewVoiceScene {
            voice: "Fin",
            music: "Nerd-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!earth",
        NewVoiceScene {
            voice: "atten",
            music: "Planet-Earth-BG-Music-1",
            playlist_folder: Some("PlanetEarth"),
        },
    ),
];

// GOALS:
//        - [x] Write obs_source to postgresql
//        - [x] Lookup obs_source in postgresql

pub async fn update_slideshow_source(
    obs_client: &OBSClient,
    source: String,
    files: Vec<SlideshowFile<'_>>,
) -> Result<()> {
    let slideshow_settings = Slideshow {
        files: &files,
        ..Default::default()
    };
    let set_settings = SetSettings {
        settings: &slideshow_settings,
        input: InputId::Name(&source),
        overlay: Some(true),
    };
    let _ = obs_client.inputs().set_settings(set_settings).await;
    Ok(())
}

pub async fn update_video_source(
    obs_client: &OBSClient,
    source: String,
    filename: String,
    looping: bool,
    close_when_inactive: bool,
) -> Result<()> {
    let settings = FfmpegSource {
        is_local_file: true,
        local_file: Path::new(&filename),
        looping,
        buffering_mb: 0,
        input: "",
        input_format: "",
        reconnect_delay_sec: 5,
        restart_on_activate: true,
        clear_on_media_end: true,
        close_when_inactive,
        speed_percent: 100,
        color_range: ColorRange::Auto,
        seekable: false,
    };
    let set_settings = SetSettings {
        settings: &settings,
        input: InputId::Name(&source),
        overlay: Some(true),
    };
    obs_client
        .inputs()
        .set_settings(set_settings)
        .await
        .map_err(|e| anyhow!("{}", e))
}

pub async fn update_image_source(
    obs_client: &OBSClient,
    source: String,
    filename: String,
) -> Result<()> {
    let image_settings = ImageSource {
        file: Path::new(&filename),
        unload: true,
    };
    let set_settings = SetSettings {
        settings: &image_settings,
        input: InputId::Name(&source),
        overlay: Some(true),
    };
    obs_client
        .inputs()
        .set_settings(set_settings)
        .await
        .map_err(|e| anyhow!("{}", e))
}

// This doesn't go here
pub async fn save_screenshot(
    client: &OBSClient,
    source: &str,
    file_path: &str,
) -> Result<()> {
    let p = Path::new(file_path);

    Ok(client
        .sources()
        .save_screenshot(SaveScreenshot {
            source: SourceId::Name(source),
            format: "png",
            file_path: p,
            width: None,
            height: None,
            compression_quality: None,
        })
        .await?)
}

// ================================================== == Scaling Sources
// ==================================================

// This takes in x & y
// plus the scene and source to scale the source
//
// the function scale() needs the id and the Scale already
// calculated to scale
pub async fn scale_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<(), anyhow::Error> {
    let id = find_id(scene, source, obs_client).await?;

    let new_scale = Scale {
        x: Some(x),
        y: Some(y),
    };

    println!(
        "\n\t#scale_source scene: {} | ID: {} | scale: {:?} {:?}",
        scene, id, new_scale.x, new_scale.y,
    );

    // This is fucking me up
    Ok(scale(scene, id, new_scale, obs_client).await?)
}

// Scale for the X & Y of the source in terms of relation to each other,
pub async fn scale(
    scene: &str,
    id: i64,
    new_scale: Scale,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene_transform = SceneItemTransform {
        scale: Some(new_scale),
        ..Default::default()
    };

    // I bet ID is wrong
    let set_transform = SetTransform {
        scene: SceneId::Name(scene),
        item_id: dbg!(id),
        transform: scene_transform,
    };
    obs_client
        .scene_items()
        .set_transform(set_transform)
        .await
        .map_err(|e| anyhow!("{}", e))
}

pub async fn old_trigger_grow(
    scene: &str,
    source: &str,
    base_scale: &Scale,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    if source == "all" {
        let sources = obs_client
            .scene_items()
            .list(SceneId::Name(&subd_types::consts::get_default_obs_source()))
            .await?;
        for source in sources {
            let new_scale = Scale {
                x: base_scale.x,
                y: base_scale.y,
            };

            // Do we need to do this not to crash all the time???
            let id = match find_id(
                &subd_types::consts::get_meme_scene(),
                &source.source_name,
                obs_client,
            )
            .await
            {
                Ok(val) => val,
                Err(_) => return Ok(()),
            };

            scale(
                &subd_types::consts::get_meme_scene(),
                id,
                new_scale,
                obs_client,
            )
            .await?;
        }
        Ok(())
    } else {
        println!("ABOUT TO SCALE SOURCE: {} {}", scene, source);
        scale_source(scene, source, x, y, obs_client).await
    }
}
// ====================================================
// == Moving Things
// ====================================================

// Move a Source using x, y and NO Filters
pub async fn move_source(
    scene: &str,
    source: &str,
    x: f32,
    y: f32,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = find_id(scene, source, obs_client).await?;

    let new_position = Position {
        x: Some(x),
        y: Some(y),
    };
    let scene_transform = SceneItemTransform {
        position: Some(new_position),
        ..Default::default()
    };

    let set_transform = SetTransform {
        scene: SceneId::Name(scene),
        item_id: id,
        transform: scene_transform,
    };
    let _ = obs_client.scene_items().set_transform(set_transform).await;
    Ok(())
}

// ====================================================
// == Hide/Show Actions
// ====================================================

pub async fn hide_source(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    set_enabled(scene, source, false, obs_client).await
}

pub async fn show_source(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    set_enabled(scene, source, true, obs_client).await
}

pub async fn hide_sources(scene: &str, obs_client: &OBSClient) -> Result<()> {
    set_enabled_on_all_sources(scene, false, obs_client).await
}

pub async fn set_enabled(
    scene: &str,
    source: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = find_id(scene, source, obs_client).await?;

    let set_enabled: obws::requests::scene_items::SetEnabled =
        obws::requests::scene_items::SetEnabled {
            enabled,
            item_id: id,
            scene: SceneId::Name(scene),
        };

    let _ = obs_client.scene_items().set_enabled(set_enabled).await;
    Ok(())
}

async fn set_enabled_on_all_sources(
    scene: &str,
    enabled: bool,
    obs_client: &OBSClient,
) -> Result<()> {
    let items = obs_client.scene_items().list(SceneId::Name(scene)).await?;
    for item in items {
        // If we can't set an item as enabled we just move on with our lives
        let _ =
            set_enabled(scene, &item.source_name, enabled, obs_client).await;
    }
    Ok(())
}

// ====================================================
// == Debug Info
// ====================================================

pub async fn print_source_info_true(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = find_id(scene, source, obs_client).await?;
    let settings = obs_client
        .scene_items()
        .transform(SceneId::Name(scene), id)
        .await?;

    println!("Source Settings: {:?}", settings);
    Ok(())
}

pub async fn print_source_info(
    source: &str,
    scene: &str,
    obs_client: &OBSClient,
) -> Result<()> {
    let id = find_id(&subd_types::consts::get_meme_scene(), source, obs_client)
        .await?;
    let settings = obs_client
        .scene_items()
        .transform(SceneId::Name(scene), id)
        .await?;

    println!("Source Settings: {:?}", settings);
    Ok(())
}

// ====================================================
// == Utility
// ====================================================

pub async fn find_id(
    scene: &str,
    source: &str,
    obs_client: &OBSClient,
) -> Result<i64> {
    let id_search = obws::requests::scene_items::Id {
        scene: SceneId::Name(scene),
        source,
        ..Default::default()
    };

    obs_client
        .scene_items()
        .id(id_search)
        .await
        .map_err(|e| anyhow!("{}", e))
}

#[derive(Debug)]
pub struct ObsSource {
    pub source: String,
    pub scene: String,
    pub position_x: sqlx::types::BigDecimal,
    pub position_y: sqlx::types::BigDecimal,
    pub scale: sqlx::types::BigDecimal,
}

pub async fn get_obs_source(
    pool: &sqlx::PgPool,
    source: String,
) -> Result<ObsSource> {
    let res =
        sqlx::query!("SELECT * FROM obs_sources WHERE source = $1", source)
            .fetch_one(pool)
            .await?;
    let model = ObsSource {
        source,
        scene: res.scene,
        position_x: res.position_x,
        position_y: res.position_y,
        scale: res.scale,
    };
    Ok(model)
}

pub async fn update_obs_source_defaults(
    pool: &sqlx::PgPool,
    source: String,
    scale: f32,
    position_x: f32,
    position_y: f32,
) -> Result<PgQueryResult> {
    let scale = BigDecimal::from_f32(scale).unwrap_or_default();
    let position_x = BigDecimal::from_f32(position_x).unwrap_or_default();
    let position_y = BigDecimal::from_f32(position_y).unwrap_or_default();
    sqlx::query!(
        r#"UPDATE obs_sources
        SET scale = $1,
        position_x = $2,
        position_y = $3
        WHERE source = $4"#,
        scale,
        position_x,
        position_y,
        source,
    )
    .execute(pool)
    .await
    .map_err(|e| anyhow!("Error updating obs_source: {}", e))
}

pub async fn update_obs_source_position(
    pool: &sqlx::PgPool,
    source: String,
    position_x: f32,
    position_y: f32,
) -> Result<PgQueryResult> {
    let position_x = BigDecimal::from_f32(position_x).unwrap_or_default();
    let position_y = BigDecimal::from_f32(position_y).unwrap_or_default();
    sqlx::query!(
        r#"UPDATE obs_sources
        SET position_x = $1,
        position_y = $2
        WHERE source = $3"#,
        position_x,
        position_y,
        source,
    )
    .execute(pool)
    .await
    .map_err(|e| anyhow!("Error updating obs_source: {}", e))
}

// We need to save:
//  - prime
//  - alex
//
//  we need to move them through chat commands
//
//  We need to write a function that reads the obs_sources values and moves
pub async fn create_obs_source(
    pool: &sqlx::PgPool,
    source: String,
    scene: String,
    scale: sqlx::types::BigDecimal,
    position_x: sqlx::types::BigDecimal,
    position_y: sqlx::types::BigDecimal,
) -> Result<PgQueryResult> {
    sqlx::query!(
        r#"INSERT INTO obs_sources(source, scene, scale, position_x, position_y)
        VALUES ( $1, $2, $3, $4, $5)"#,
        source,
        scene,
        scale,
        position_x,
        position_y,
    )
    .execute(pool)
    .await
    .map_err(|e| anyhow!("Error saving obs_source: {}", e))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use subd_db::get_db_pool;
//
//     #[tokio::test]
//     async fn test_obs_sources() {
//         let pool = get_db_pool().await;
//         let source = "technofroggo".to_string();
//         let scene = "Memes".to_string();
//         let scale = 0.3;
//
//         let position_x = 100.0;
//         let position_y = 100.0;
//         let x = BigDecimal::from_f32(position_x).unwrap();
//         let y = BigDecimal::from_f32(position_y).unwrap();
//         let scale = BigDecimal::from_f32(scale).unwrap();
//
//         let res =
//             create_obs_source(&pool, source.clone(), scene, scale, x, y).await;
//         if let Err(e) = res {
//             println!("Error: {}", e);
//         }
//
//         // let _ = save_obs_source(
//         //     &pool,
//         //     source.to_string(),
//         //     scale.into(),
//         //     position_x.into(),
//         //     position_y.into(),
//         // )
//         // .await;
//
//         // let res = get_obs_source(&pool, source.to_string()).await;
//         // dbg!(&res);
//
//         // We need to look up and move
//     }
// }
