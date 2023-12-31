use crate::art_blocks;
use crate::dalle;
use crate::image_generation::GenerateImage;
use crate::obs;
use crate::obs_source;
use crate::stable_diffusion;
use crate::move_transition;
use crate::move_transition_effects;
use crate::stable_diffusion::StableDiffusionRequest;
use anyhow::Result;
use chrono::Utc;
use obws::Client as OBSClient;
use subd_types::{Event, UserMessage};
use tokio::sync::broadcast;
use std::thread;
use std::time::Duration;

pub async fn handle_stream_background_commands(
    _tx: &broadcast::Sender<Event>,
    obs_client: &OBSClient,
    splitmsg: Vec<String>,
    msg: UserMessage,
) -> Result<()> {
    let _is_mod = msg.roles.is_twitch_mod();
    let _is_vip = msg.roles.is_twitch_vip();
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

        // !move bogan -350 600 9000
        // - Move current bogan down off screen
        // - Hide cauldron
        // - Wait for image to complete
        // - Show cauldron
        // - Play lightnting / thunder
        // - Trigger slow rise

        "!bogan" => {
            let scene = "AIAssets";
            let source = "bogan";
            let req = move_transition::ChatMoveSourceRequest {
                ..Default::default()
            };
            
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
            let prompt = splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");

            let prompt = if screenshot_source == "begin-base" {
                format!("{}. on a bright chroma key green background", prompt)
            } else {
                prompt
            };

            
            let path = stable_diffusion::create_image_variation(
                prompt,
                filename,
                unique_identifier,
                None,
                false,
            )
            .await?;
            
            // Hide the Last Bogan
            let _ = move_transition_effects::move_source_in_scene_x_and_y(
                scene,
                source,
                -300.0,
                1100.0,
                0,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await;
            // do we need to sleep here?
            thread::sleep(Duration::from_millis(100));

            let source = obs::NEW_BEGIN_SOURCE.to_string();
            let res = obs_source::update_image_source(
                obs_client,
                source.clone(),
                path,
            )
            .await;
            if let Err(e) = res {
                eprintln!("Error Updating OBS Source: {} - {}", source, e);
            };
            
            let _ = move_transition_effects::move_source_in_scene_x_and_y(
                &scene,
                &source,
                -300.0,
                600.0,
                6000,
                req.easing_function_index,
                req.easing_type_index,
                &obs_client,
            )
            .await;
            Ok(())
        }

        "!picasso" | "!sd" => {
            // if not_beginbot {
            //     return Ok(());
            // }
            //
            let prompt = splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");

            let req = stable_diffusion::StableDiffusionRequest {
                prompt: prompt.clone(),
                username: msg.user_name,
                amount: 1,
            };
            // We need to finish the code though
            let _ = req.generate_image(prompt, None, true).await;
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
