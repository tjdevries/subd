use crate::art_blocks;
use crate::dalle;
use crate::image_generation::GenerateImage;
use crate::move_transition_effects;
use crate::obs;
use crate::obs_source;
use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use obws::Client as OBSClient;
use stable_diffusion;
use stable_diffusion::models::GenerateAndArchiveRequest;
use stable_diffusion::models::RequestType;
use stable_diffusion::models::RequestType::Img2ImgFile;
use stable_diffusion::run_from_prompt;
use stable_diffusion::stable_diffusion_from_image;
use std::thread;
use std::time::Duration;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;

pub async fn handle_stream_background_commands(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _is_mod = msg.roles.is_twitch_mod();
    let _is_vip = msg.roles.is_twitch_vip();
    let is_sub = msg.roles.is_twitch_sub();

    let not_beginbot =
        msg.user_name != "beginbot" && msg.user_name != "beginbotbot";

    let command = splitmsg[0].as_str();

    match command {
        "!ab" => {
            let ab_id = &splitmsg[1];
            let ab_url = format!(
                "https://generator.artblocks.io/0x99a9b7c1116f9ceeb1652de04d5969cce509b069/{}",
                ab_id);
            let browser_settings =
                obws::requests::custom::source_settings::BrowserSource {
                    url: ab_url.as_ref(),
                    ..Default::default()
                };
            let set_settings = obws::requests::inputs::SetSettings {
                settings: &browser_settings,
                input: "AB-Browser",
                overlay: Some(true),
            };
            let _ = obs_client.inputs().set_settings(set_settings).await;
            Ok(())
        }

        "!dca" | "!heart" => {
            let lower_bound = 1;
            let upper_bound = 1000;
            let contract = "0x77d4b54e91822e9799ab0900876d6b1cda752706";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!axo" | "!jiwa" => {
            let lower_bound = 480000000;
            let upper_bound = 480000399;
            let contract = "0x99a9b7c1116f9ceeb1652de04d5969cce509b069";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!chimera" => {
            let lower_bound = 233000000;
            let upper_bound = 233000986;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!watercolor" => {
            let lower_bound = 59000000;
            let upper_bound = 59000599;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!pig" => {
            let lower_bound = 129000000;
            let upper_bound = 129001023;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!run" => {
            let lower_bound = 138000000;
            let upper_bound = 138000999;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!vortex" | "!v" => {
            let lower_bound = 225000000;
            let upper_bound = 225000999;
            let contract = "0xa7d8d9ef8d8ce8992df33d8b8cf4aebabd5bd270";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!memories" | "!m" => {
            let lower_bound = 428000000;
            let upper_bound = 428000449;
            let contract = "0x99a9b7c1116f9ceeb1652de04d5969cce509b069";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        "!steviep" | "!dopamine" | "!d" => {
            let lower_bound = 457000000;
            let upper_bound = 457000776;
            let contract = "0x99a9b7c1116f9ceeb1652de04d5969cce509b069";
            let _ = art_blocks::updates_ab_browser(
                &obs_client,
                contract.to_string(),
                lower_bound,
                upper_bound,
            )
            .await;
            Ok(())
        }

        // !bogan 0.77 A Description
        //
        // This uses default strength
        // !bogan A Description
        "!bogan" => {
            if !is_sub {
                return Ok(());
            }
            if let Err(e) = create_and_show_bogan(obs_client, splitmsg).await {
                eprintln!("Error Creating Bogan: {}", e);
            }
            Ok(())
        }

        "!picasso" | "!sd" => {
            let prompt = splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");

            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let unique_identifier = format!("{}_screenshot.png", timestamp);
            let req = GenerateAndArchiveRequest {
                prompt: prompt.clone(),
                unique_identifier,
                request_type: RequestType::Prompt2Img,
                set_as_obs_bg: true,
                additional_archive_dir: None,
                strength: None,
            };
            println!("Running Stable D");

            let _ = run_from_prompt(&req).await;
            Ok(())
        }

        "!dalle" => {
            if not_beginbot {
                return Ok(());
            }

            let prompt = splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");

            println!("Dalle Time!");

            let req = dalle::DalleRequest {
                prompt: prompt.clone(),
                username: msg.user_name,
                amount: 1,
            };

            let _ = req.generate_image(prompt, None, true).await;
            Ok(())
        }
        _ => Ok(()),
    }
}

async fn create_and_show_bogan(
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
) -> Result<()> {
    let duration = 6000;
    let easing_function_index = 0;
    let easing_type_index = 0;
    let end_pos = (1958.0, 449.0);
    let scene = "AIAssets";
    let source = "bogan";

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_screenshot.png", timestamp);

    // TODO: Update this
    // This is wierd
    let filename = format!(
        // "./tmp/screenshots/timelapse/{}",
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
    // is this strength string parseable to a f32?
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

    let _ = move_transition_effects::move_source_in_scene_x_and_y(
        scene,
        source,
        end_pos.0,
        end_pos.1 + 500.0,
        0,
        easing_function_index,
        easing_type_index,
        &obs_client,
    )
    .await;
    // do we need to sleep here?
    thread::sleep(Duration::from_millis(100));

    let source = obs::NEW_BEGIN_SOURCE.to_string();
    let res =
        obs_source::update_image_source(obs_client, source.clone(), path).await;
    if let Err(e) = res {
        eprintln!("Error Updating OBS Source: {} - {}", source, e);
    };
    move_transition_effects::move_source_in_scene_x_and_y(
        &scene,
        &source,
        end_pos.0,
        end_pos.1,
        duration,
        easing_function_index,
        easing_type_index,
        &obs_client,
    )
    .await
}
