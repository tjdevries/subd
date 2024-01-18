use crate::constants;
use crate::move_transition::move_source::CropSettings;
use crate::move_transition::move_transition;
use crate::obs::obs_source;
use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use obws::Client as OBSClient;
use stable_diffusion;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;

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
        path,
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
    move_transition::move_source(
        SCENE,
        SOURCE,
        filter_name,
        Some(-580.0),
        Some(-700.0),
        Some(c),
        obs_client,
    )
    .await
}

// let filter_enabled = obws::requests::filters::SetEnabled {
//     source,
//     filter: &filter_name,
//     enabled: true,
// };
// Ok(obs_client.filters().set_enabled(filter_enabled).await?)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::obs::create_obs_client;
    // use crate::move_transition::duration;
    // use crate::move_transition::models::Coordinates;

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
