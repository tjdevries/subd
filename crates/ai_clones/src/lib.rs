use anyhow::Result;
use obs_move_transition::move_source;
use obws::requests::custom::source_settings::ImageSource;
use obws::requests::inputs::Create;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;
use std::path::Path;

pub mod bogan_position;
pub mod chat;
pub mod utils;

// TODO move this to somewhere else / pull in from config
const SCREENSHOT_SOURCE: &str = "begin-base";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChromaKey {
    pub similarity: i32,
}

pub async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    println!("Parsing Args");
    let (prompt, strength) =
        chat::parse_args(&splitmsg, SCREENSHOT_SOURCE.to_string())?;

    println!("Taking Screenshot");
    let (filename, unique_identifier) =
        utils::take_screenshot(SCREENSHOT_SOURCE.to_string(), obs_client)
            .await?;

    println!("Generating Screenshot Variation w/ {}", prompt.clone());
    let req = stable_diffusion::models::GenerateAndArchiveRequestBuilder::new(
        prompt.clone(),
        unique_identifier,
        Img2ImgFile(filename),
    )
    .strength(strength.unwrap_or(0.4))
    .build();
    let path = stable_diffusion_from_image(&req).await?;

    let new_begin_source =
        subd_types::consts::get_ai_twin_obs_source().to_string();
    let res = obs_service::obs_source::update_image_source(
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

    recruit_new_bogan_member(path, obs_client).await
}

async fn recruit_new_bogan_member(
    path: String,
    obs_client: &OBSClient,
) -> Result<()> {
    let scene = "BoganArmy";
    let (new_source, index) =
        create_new_bogan_source(scene, path.clone(), obs_client).await?;

    println!("Creating Move Source: {} {}", scene, new_source);
    // We have a Chroma key on the whole scene, instead of adding it to every single source
    // create_chroma_key_filter(&new_source, &obs_client).await?;

    // Do we have to call this???
    let _ = obs_service::obs_source::update_image_source(
        obs_client,
        new_source.clone(),
        path,
    )
    .await;

    let _ = create_move_source_filter(scene, &new_source, obs_client).await;

    let _ = bogan_position::rotate_bogan_order(scene, index, obs_client).await;

    // Use the index to move!
    // index - 3 = hide
    // index - 2 = position_4
    // index - 1 = position_3
    // pos: x+-580.0 y+0.0 rot:+0.0 scale: x*0.300 y*0.300 crop: l 580 t 0 r 0 b 0
    // pos: x 359.0 y 921.0 rot: 0.0 scale: x 0.211 y 0.211 crop: l 580 t 0 r 0 b 0

    Ok(())
}

// This needs to go in some other generic OBS filters file
async fn _create_chroma_key_filter(
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
        source: scene,
        filter: &filter_name,
        kind: &subd_types::consts::get_move_source_filter_kind(),
        settings: Some(move_source_settings),
    };
    Ok(obs_client.filters().create(new_filter).await?)
}

pub async fn find_current_bogan_index(
    scene: &str,
    obs_client: &OBSClient,
) -> Result<i32> {
    let res = obs_client.scene_items().list(scene).await.unwrap();
    Ok(res
        .iter()
        .map(|x| parse_scene_item_index(&x.source_name))
        .max()
        .unwrap_or(0))
}

async fn create_new_bogan_source(
    scene: &str,
    path: String,
    obs_client: &OBSClient,
) -> Result<(String, i32)> {
    let index = find_current_bogan_index(scene, obs_client).await? + 1;
    let new_source = format!("bogan_{}", index);
    println!("Creating Scene Item: {}", new_source);

    let settings = ImageSource {
        file: Path::new(&path),
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
    Ok((new_source, index))
}

fn parse_scene_item_index(scene_item: &str) -> i32 {
    let v: Vec<&str> = scene_item.split('_').collect();
    let index = v.get(1).unwrap_or(&"0");
    index.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obs_service::obs;

    #[tokio::test]
    #[ignore]
    async fn test_bogan_position() {
        let obs_client = obs::create_obs_client().await.unwrap();

        let scene = "BoganArmy";
        let _index =
            find_current_bogan_index(scene, &obs_client).await.unwrap();

        let _bogan_1 = "bogan_13";
        let _bogan_2 = "bogan_14";
        let _bogan_3 = "bogan_15";
        let _bogan_4 = "bogan_16";
    }
}
