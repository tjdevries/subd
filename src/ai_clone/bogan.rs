use serde::{Deserialize, Serialize};
use crate::constants;
use crate::move_transition::models::Coordinates;
use crate::move_transition::move_source;
use crate::move_transition::move_source::CropSettings;
use crate::move_transition::move_transition;
use crate::obs::obs_source;
use anyhow::Result;
use obws::requests::custom::source_settings::ImageSource;
use obws::requests::inputs::Create;
use obws::Client as OBSClient;
use std::path::Path;
use stable_diffusion;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;
use crate::ai_clone::chat;
use crate::ai_clone::utils;

// TODO move this to somewhere else / pull in from config
const SCREENSHOT_SOURCE: &str = "begin-base";
const SCENE: &str = "AIAssets";
const SOURCE: &str = "bogan";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChromaKey {
    similarity: i32,
}


// pub mod something
pub async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    let (prompt, strength) =
        chat::parse_args(&splitmsg, SCREENSHOT_SOURCE.to_string())?;

    let (filename, unique_identifier) =
        utils::take_screenshot(SCREENSHOT_SOURCE.to_string(), &obs_client).await?;

    let req = stable_diffusion::models::GenerateAndArchiveRequestBuilder::new(
        prompt.clone(),
        unique_identifier,
        Img2ImgFile(filename),
    )
    .strength(strength.unwrap_or(0.4))
    .build();

    println!("Generating Screenshot Variation w/ {}", prompt.clone());
    let path = stable_diffusion_from_image(&req).await?;

    let new_begin_source = constants::NEW_BEGIN_SOURCE.to_string();
    let res = obs_source::update_image_source(
        obs_client,
        new_begin_source.clone(),
        path.clone(),
    )
    .await;
    if let Err(e) = res {
        eprintln!(
            "Error Updating OBS new_begin_source: {} - {}",
            new_begin_source, e
        );
    };

    let c = CropSettings::builder().left(580.0).build();
    let filter_name = format!("Move_{}", SOURCE);
    let _ = move_transition::move_source(
        SCENE,
        SOURCE,
        filter_name,
        Some(-580.0),
        Some(-700.0),
        Some(c),
        None,
        obs_client,
    )
    .await;

    recruit_new_bogan_member(path, &obs_client).await
}



async fn recruit_new_bogan_member(
    path: String,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene = "BoganArmy";
    let res = obs_client.scene_items().list(scene).await.unwrap();
    let index = res
        .iter()
        .map(|x| parse_scene_item_index(&x.source_name))
        .max()
        .unwrap()
        + 1;
    let new_source = format!("bogan_{}", index);
    println!("Creating Scene Item: {}", new_source);

    let settings = ImageSource {
        file: &Path::new(&path),
        unload: true,
    };
    let c = Create {
        scene,
        input: &new_source,
        kind: "image_source",
        settings: Some(settings),
        enabled: Some(true),
    };
    if let Err(e) = obs_client.inputs().create(c).await {
        eprintln!("Error creating input: {}", e);
    };
    let chroma_key = ChromaKey { similarity: 420 };
    let new_filter = obws::requests::filters::Create {
        source: &new_source,
        filter: "Chroma Key",
        kind: "chroma_key_filter_v2",
        settings: Some(chroma_key),
    };
    obs_client.filters().create(new_filter).await.unwrap();

    let _ =
        obs_source::update_image_source(&obs_client, new_source.clone(), path)
            .await;

    let move_source_settings =
        move_source::MoveSourceSettingsBuilder::new().build();

    let filter_name = format!("Move_{}", new_source);
    let new_filter: obws::requests::filters::Create<
        move_source::MoveSourceSettings,
    > = obws::requests::filters::Create {
        source: &scene,
        filter: &filter_name,
        kind: constants::MOVE_SOURCE_FILTER_KIND,
        settings: Some(move_source_settings),
    };
    let _ = obs_client.filters().create(new_filter).await;

    let scale = Coordinates::new(Some(0.3), Some(0.3));
    let c = CropSettings::builder().left(580.0).build();
    let filter_name = format!("Move_{}", SOURCE);
    let _ = move_transition::move_source(
        scene,
        new_source,
        filter_name,
        Some(-580.0),
        None,
        Some(c),
        Some(scale),
        obs_client,
    )
    .await;

    Ok(())
}

fn parse_scene_item_index(scene_item: &str) -> i32 {
    let v: Vec<&str> = scene_item.split("_").collect();
    let index = v.get(1).unwrap_or(&"0");
    index.parse().unwrap_or(0)
}
