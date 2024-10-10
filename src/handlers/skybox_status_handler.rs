extern crate reqwest;
use askama::Template;
extern crate serde;
extern crate serde_json;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use events::EventHandler;
use obs_service::obs_source;
use obws::Client as OBSClient;
use skybox;
use skybox::skybox_requests;
use std::fs::File;
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

static SKYBOX_SCENE: &str = "SkyboxScene";
static SKYBOX_SOURCE: &str = "skybox";

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for SkyboxStatusHandler {
    async fn handle(
        self: Box<Self>,
        _tx: broadcast::Sender<Event>,
        _rx: broadcast::Receiver<Event>,
    ) -> Result<()> {
        loop {
            self.process_uncompleted_requests().await?;
            sleep(Duration::from_secs(5)).await;
        }
    }
}

impl SkyboxStatusHandler {
    async fn process_uncompleted_requests(&self) -> Result<()> {
        let uncompleted_reqs =
            skybox_requests::uncompleted_skybox_requests(&self.pool).await?;
        for req in uncompleted_reqs {
            self.process_request(req).await?;
        }
        Ok(())
    }

    async fn process_request(
        &self,
        req: skybox_requests::skybox_request::Model,
    ) -> Result<()> {
        match skybox::check_skybox_status(req.blockade_id).await {
            Ok(skybox_status) => {
                self.handle_skybox_status(req, skybox_status).await?
            }
            Err(_e) => println!("Err(skybox_status)"),
        }
        Ok(())
    }

    async fn handle_skybox_status(
        &self,
        req: skybox_requests::skybox_request::Model,
        skybox_status: skybox::SkyboxStatusResponse,
    ) -> Result<()> {
        let file_url = skybox_status.file_url;
        println!("\tFind a skybox_status: {:?}", file_url);

        if !file_url.is_empty() {
            self.update_request(&req, &file_url).await?;
            self.save_image(&req, &file_url).await?;
            self.update_skybox_template(&file_url).await?;
            self.update_obs_source().await?;
        }
        Ok(())
    }

    async fn update_request(
        &self,
        req: &skybox_requests::skybox_request::Model,
        file_url: &str,
    ) -> Result<()> {
        let completed_at = sqlx::types::time::OffsetDateTime::now_utc();
        skybox_requests::update_skybox_request(
            &self.pool,
            req.blockade_id,
            file_url.to_string(),
            completed_at,
        )
        .await?;
        Ok(())
    }

    async fn save_image(
        &self,
        req: &skybox_requests::skybox_request::Model,
        file_url: &str,
    ) -> Result<()> {
        let image_data = reqwest::get(file_url).await?.bytes().await?.to_vec();
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let unique_identifier = format!("{}_{}", timestamp, req.blockade_id);
        let archive_file =
            format!("./archive/skybox/{}.png", unique_identifier);
        let mut file = File::create(archive_file)?;
        file.write_all(&image_data)?;
        println!("After Save the Raw Image");
        Ok(())
    }

    async fn update_skybox_template(&self, file_url: &str) -> Result<()> {
        let skybox_template = skybox::SkyboxTemplate { url: file_url };
        let new_skybox = "/home/begin/code/subd/build/skybox.html";
        let mut file = File::create(new_skybox)?;
        let render = skybox_template.render()?;
        file.write_all(render.as_bytes())?;
        Ok(())
    }

    async fn update_obs_source(&self) -> Result<()> {
        obs_source::hide_source(SKYBOX_SCENE, SKYBOX_SOURCE, &self.obs_client)
            .await?;
        sleep(Duration::from_secs(1)).await;
        obs_source::show_source(SKYBOX_SCENE, SKYBOX_SOURCE, &self.obs_client)
            .await?;
        Ok(())
    }
}
