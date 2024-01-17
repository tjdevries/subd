use crate::move_transition::duration;
use crate::move_transition::move_transition;
use crate::obs::obs_source;
use crate::constants;
use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use obws::Client as OBSClient;
use stable_diffusion;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::stable_diffusion_from_image;

// pub mod something
pub async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    let duration = 6000;
    let end_pos = (1958.0, 449.0);
    let scene = "AIAssets";
    let source = "bogan";

    // TODO: Update this
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);
    let filename = format!(
        "/home/begin/code/subd/tmp/screenshots/{}",
        unique_identifier
    );

    // TODO: Extract this OBS
    let screenshot_source = "begin-base";
    if let Err(e) =
        obs_source::save_screenshot(&obs_client, screenshot_source, &filename)
            .await
    {
        eprintln!("Error Saving Screenshot: {}", e);
        return Ok(());
    };
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

    // TODO: this should be new
    let req = stable_diffusion::models::GenerateAndArchiveRequest {
        prompt: prompt.clone(),
        unique_identifier,
        request_type: Img2ImgFile(filename),
        set_as_obs_bg: false,
        additional_archive_dir: None,
        strength,
    };
    println!("Generating Screenshot Variation w/ {}", prompt.clone());
    let path = stable_diffusion_from_image(&req).await?;

    let d = duration::EasingDuration::new(duration);
    let _ = move_transition::move_source_in_scene_x_and_y(
        &obs_client,
        scene,
        source,
        end_pos.0,
        end_pos.1 + 500.0,
        d,
    )
    .await;
    let filter_name = format!("Move_{}", source);

    let _ = move_transition::move_source(
        scene,
        source,
        filter_name,
        Some(-100.0),
        Some(-100.0),
        &obs_client,
    )
    .await;

    let source = constants::NEW_BEGIN_SOURCE.to_string();
    let res =
        obs_source::update_image_source(obs_client, source.clone(), path).await;
    if let Err(e) = res {
        eprintln!("Error Updating OBS Source: {} - {}", source, e);
    };
    let d = duration::EasingDuration::new(duration);
    move_transition::move_source_in_scene_x_and_y(
        &obs_client,
        &scene,
        &source,
        end_pos.0,
        end_pos.1,
        d,
    )
    .await
}
