extern crate reqwest;
use askama::Template;
extern crate serde;
extern crate serde_json;
use crate::obs_source;
use crate::skybox;
use crate::skybox_requests;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obws::Client as OBSClient;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Write;
use std::time::Duration;
use subd_types::Event;
use tokio;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub struct SkyboxStatusHandler {
    pub obs_client: OBSClient,
    pub pool: sqlx::PgPool,
}

// This should be moved into a handlers/ folder
#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxStatusHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        let mut loop_count = 0;

        loop {
            // println!("Status Loop: {}", loop_count);
            loop_count += 1;

            // we must be unwrapping and crashing at some point
            let uncompleted_reqs =
                skybox_requests::uncompleted_skybox_requests(&self.pool)
                    .await
                    .unwrap();

            for req in uncompleted_reqs {
                // println!("Inside req");
                match skybox::check_skybox_status(req.blockade_id).await {
                    Ok(skybox_status) => {
                        let file_url = skybox_status.file_url;

                        println!("\tFind a skybox_status: {:?}", file_url);

                        if file_url != "" {
                            let completed_at =
                                sqlx::types::time::OffsetDateTime::now_utc();

                            let _ = skybox_requests::update_skybox_request(
                                &self.pool,
                                req.blockade_id,
                                file_url.clone(),
                                completed_at,
                            )
                            .await;

                            // println!("Save the Raw Image");
                            let image_data = reqwest::get(file_url.clone())
                                .await?
                                .bytes()
                                .await?
                                .to_vec();
                            let timestamp =
                                Utc::now().format("%Y%m%d%H%M%S").to_string();
                            let unique_identifier =
                                format!("{}_{}", timestamp, req.blockade_id);
                            let archive_file = format!(
                                "./archive/skybox/{}.png",
                                unique_identifier
                            );
                            let mut file = File::create(archive_file).unwrap();
                            file.write_all(&image_data).unwrap();
                            println!("After Save the Raw Image");

                            let skybox_template =
                                skybox::SkyboxTemplate { url: &file_url };

                            let new_skybox =
                                "/home/begin/code/subd/build/skybox.html";
                            let mut file = File::create(new_skybox).unwrap();
                            let render = skybox_template.render().unwrap();
                            file.write_all(render.as_bytes()).unwrap();

                            let scene = "Primary";
                            let source = "skybox";

                            let _ = obs_source::hide_source(
                                scene,
                                source,
                                &self.obs_client,
                            )
                            .await;
                            sleep(Duration::from_secs(1)).await;
                            let _ = obs_source::show_source(
                                scene,
                                source,
                                &self.obs_client,
                            )
                            .await;
                        }
                    }
                    Err(_e) => {
                        println!("Err(skybox_status)");
                    }
                };
            }

            // println!("Sleep Time!");

            sleep(Duration::from_secs(60)).await;
        }
    }
}
