use crate::art_blocks;
use crate::dalle;
use anyhow::Result;
use obws::Client as OBSClient;
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
    let _not_beginbot =
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

        "!dalle" => {
            let prompt = splitmsg
                .iter()
                .skip(1)
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join(" ");
            println!("Dalle Time!");
            let _ = dalle::dalle_time(prompt, msg.user_name, 1).await;
            Ok(())
        }
        _ => Ok(()),
    }
}
