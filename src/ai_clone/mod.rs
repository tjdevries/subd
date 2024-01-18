use crate::constants;
use crate::move_transition::models::Coordinates;
use crate::move_transition::move_source;
use crate::move_transition::move_source::CropSettings;
use crate::move_transition::move_transition;
use crate::obs::obs_source;
use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use obws::requests::custom::source_settings::ImageSource;
use obws::requests::inputs::Create;
use obws::Client as OBSClient;
use serde::Deserialize;
use serde::Serialize;
use stable_diffusion;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;
use std::path::Path;

fn parse_args(
    splitmsg: &Vec<String>,
    screenshot_source: String,
) -> Result<(String, Option<f32>)> {
    // TODO: Extract this to a constant or config

    let strength = splitmsg.get(1).ok_or(anyhow!("Nothing to modify!"))?;
    let parsed_strength = strength.parse::<f32>();

    let (prompt_offset, strength) = match parsed_strength {
        Ok(f) => (2, Some(f)),
        Err(_) => (1, None),
    };

    let prompt = splitmsg
        .iter()
        .skip(prompt_offset)
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>()
        .join(" ");

    let prompt = if screenshot_source == "begin-base" {
        format!("{}. on a bright chroma key green screen background", prompt)
    } else {
        prompt
    };

    Ok((prompt, strength))
}

// TODO: Update this
async fn take_screenshot(
    screenshot_source: String,
    obs_client: &OBSClient,
) -> Result<(String, String)> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename = format!(
        "/home/begin/code/subd/tmp/screenshots/{}",
        unique_identifier
    );
    obs_source::save_screenshot(&obs_client, &screenshot_source, &filename)
        .await
        .map(|_| (filename, unique_identifier))
}

// TODO move this to somewhere else / pull in from config
const SCREENSHOT_SOURCE: &str = "begin-base";
const SCENE: &str = "AIAssets";
const SOURCE: &str = "bogan";

// pub mod something
pub async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    let (prompt, strength) =
        parse_args(&splitmsg, SCREENSHOT_SOURCE.to_string())?;

    let (filename, unique_identifier) =
        take_screenshot(SCREENSHOT_SOURCE.to_string(), &obs_client).await?;

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

// let filter_enabled = obws::requests::filters::SetEnabled {
//     source,
//     filter: &filter_name,
//     enabled: true,
// };
// Ok(obs_client.filters().set_enabled(filter_enabled).await?)

// pub struct SceneItem {
//     id: i32,
//     index: i32,
//     source_name: String,
//
//
// }

// &res = [
//     SceneItem {
//         id: 1,
//         index: 0,
//         source_name: "tes11",
//         source_type: Input,
//         input_kind: Some(
//             "image_source",
//         ),
//         is_group: None,
//     },
// ]

fn parse_scene_item_index(scene_item: &str) -> i32 {
    let v: Vec<&str> = scene_item.split("_").collect();
    let index = v.get(1).unwrap_or(&"0");
    index.parse().unwrap_or(0)
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChromaKey {
    similarity: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs::create_obs_client;

    #[tokio::test]
    async fn test_bogan_army_parsing() {
        let obs_client = create_obs_client().await.unwrap();
        let filter = "Chroma Key";
        let source = "bogan_12";
        let filter_details =
            obs_client.filters().get("bogan_12", filter).await.unwrap();
        let res = ::serde_json::to_string_pretty(&filter_details).unwrap();
        println!("res: {:?}", &res);

        let chroma_key = ChromaKey { similarity: 420 };
        let new_filter = obws::requests::filters::Create {
            source,
            filter: "chroma_key_filter_v2",
            kind: constants::STREAM_FX_INTERNAL_FILTER_NAME,
            settings: Some(chroma_key),
        };
        obs_client.filters().create(new_filter).await.unwrap();

        // let scene_item = "bogan_11";
        // let res = parse_scene_item_index(scene_item);
        // assert_eq!(11, res);
    }
    // use crate::move_transition::duration;
    // use crate::move_transition::models::Coordinates;

    // Read all sources from scene
    // Split all on index
    // Find next index
    // Create new Source (Image Source) with the next index
    #[tokio::test]
    async fn test_bogan_army() {
        let obs_client = create_obs_client().await.unwrap();

        // recruit_new_bogan_memeber(path, obs_client).await

        // let filter_name = format!("3D-Transform-{}", camera_type);
        // let stream_fx_settings = StreamFXSettings {
        //     camera_mode: Some(i as i32),
        //     ..Default::default()
        // };
        // let new_filter = obws::requests::filters::Create {
        //     source,
        //     filter: &filter_name,
        //     kind: constants::STREAM_FX_INTERNAL_FILTER_NAME,
        //     settings: Some(stream_fx_settings),
        // };
        // obs_client.filters().create(new_filter).await?;

        // println!("res: {:?}", res);
        // dbg!(&res);
        //k obs_client.scenes().list()

        // obs_client.scenes().set_current_scene("BoganArmy").await.unwrap();
    }

    #[tokio::test]
    async fn test_bogan() {
        let splitmsg: Vec<String> = vec![
            "!bogan".to_string(),
            "disney".to_string(),
            "cat".to_string(),
        ];
        let obs_client = create_obs_client().await.unwrap();
        if let Err(e) = create_and_show_bogan(&obs_client, splitmsg).await {
            eprintln!("Error creating bogan: {}", e);
        }

        // Just enable a filter
    }
    #[tokio::test]
    async fn test_army_of_bogans() {
        let _obs_client = create_obs_client().await.unwrap();
        let scene = "BoganArmy";
        let source = "Bogan1";

        let _b = obws::requests::scene_items::CreateSceneItem {
            scene,
            source,
            enabled: Some(true),
        };
    }
}
