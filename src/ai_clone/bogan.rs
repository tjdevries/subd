use crate::ai_clone::chat;
use crate::ai_clone::utils;
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
use serde::{Deserialize, Serialize};
use stable_diffusion;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;
use std::path::Path;

// TODO move this to somewhere else / pull in from config
const SCREENSHOT_SOURCE: &str = "begin-base";
const SCENE: &str = "AIAssets";
const SOURCE: &str = "bogan";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChromaKey {
    pub similarity: i32,
}

pub async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    let (prompt, strength) =
        chat::parse_args(&splitmsg, SCREENSHOT_SOURCE.to_string())?;

    let (filename, unique_identifier) =
        utils::take_screenshot(SCREENSHOT_SOURCE.to_string(), &obs_client)
            .await?;

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
    let new_source =
        create_new_bogan_source(scene.clone(), path.clone(), obs_client)
            .await?;

    println!("Creating Move Source: {} {}", scene, new_source);
    create_chroma_key_filter(&new_source, &obs_client).await?;

    // Do we have to call this???
    let _ =
        obs_source::update_image_source(&obs_client, new_source.clone(), path)
            .await;

    let _ = create_move_source_filter(&scene, &new_source, obs_client).await;

    // pos: x+-580.0 y+0.0 rot:+0.0 scale: x*0.300 y*0.300 crop: l 580 t 0 r 0 b 0
    // pos: x 359.0 y 921.0 rot: 0.0 scale: x 0.211 y 0.211 crop: l 580 t 0 r 0 b 0

    // This is where we are trying to scale and crop our source
    let scale = Coordinates::new(Some(0.2), Some(0.2));
    let c = CropSettings::builder().left(580.0).build();
    let filter_name = format!("Move_{}", new_source);

    let x = 359.0;
    let y = 921.0;

    if let Err(e) = move_transition::move_source(
        scene,
        new_source.clone(),
        filter_name.clone(),
        Some(x),
        Some(y),
        Some(c),
        Some(scale),
        obs_client,
    )
    .await
    {
        // Error moving source: bogan_84 in scene BoganArmy with filter Move_bogan - Failed to update Filter: Move_bogan on So
        // urce: BoganArmy
        eprintln!(
            "Error moving source: {} in scene {} with filter {} - {}",
            new_source, scene, filter_name, e,
        );
    };
    Ok(())
}

// This needs to go in some other generic OBS filters file
async fn create_chroma_key_filter(
    source: &String,
    obs_client: &OBSClient,
) -> Result<()> {
    let chroma_key = ChromaKey { similarity: 420 };
    let new_filter = obws::requests::filters::Create {
        source,
        filter: "Chroma Key",
        kind: "chroma_key_filter_v2",
        settings: Some(chroma_key),
    };
    Ok(obs_client.filters().create(new_filter).await?)
}

async fn create_move_source_filter(
    scene: &str,
    source: &String,
    obs_client: &OBSClient,
) -> Result<()> {
    let move_source_settings =
        move_source::MoveSourceSettingsBuilder::new(source.to_string()).build();

    let filter_name = format!("Move_{}", source);

    // How is this on the right scene
    let new_filter: obws::requests::filters::Create<
        move_source::MoveSourceSettings,
    > = obws::requests::filters::Create {
        source: &scene,
        filter: &filter_name,
        kind: constants::MOVE_SOURCE_FILTER_KIND,
        settings: Some(move_source_settings),
    };
    Ok(obs_client.filters().create(new_filter).await?)
}

async fn create_new_bogan_source(
    scene: &str,
    path: String,
    obs_client: &OBSClient,
) -> Result<String> {
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
    obs_client.inputs().create(c).await?;
    Ok(new_source)
}

fn parse_scene_item_index(scene_item: &str) -> i32 {
    let v: Vec<&str> = scene_item.split("_").collect();
    let index = v.get(1).unwrap_or(&"0");
    index.parse().unwrap_or(0)
}
