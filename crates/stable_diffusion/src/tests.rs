use anyhow::Result;
use crate::models;
use crate::service;

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_process_stable_diffusion() -> Result<()> {
        let req = models::GenerateAndArchiveRequest {
            ..Default::default()
            // prompt: "batman".to_string(),
            // username: "beginbot".to_string(),
            // amount: -2,
        };
       Ok(()) 
    }

    #[test]
    fn test_parsing_carls_images() {
        let filepath = "./archive/b.json";
        // let srcdir = PathBuf::from(filepath);
        let f = fs::canonicalize(filepath).unwrap();

        let mut file = File::open(f).unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let res: SDResponse = serde_json::from_str(&contents).unwrap();
        let base64 = &res.data[0].b64_json;
        let bytes = general_purpose::STANDARD.decode(base64).unwrap();

        // We actually don't want to be running this, cuz it actually runs and saves
        // We need a good name for this
        // let mut file =
        //     File::create("durf2.png").expect("Failed to create file");
        // file.write_all(&bytes).expect("Failed to write to file");
        //
        // // Unless it's none
        // let _content = &res.choices[0].message.content;

        // assert_eq!(srcdir.to_string(), "".to_string());
    }
   
    #[tokio::test]
    async fn test_stable_d() -> Result<()> {
        let prompt = "batman".to_string();
        let username = "beginbot".to_string();
        let req = models::StableDiffusionRequest{
            prompt: prompt.clone(),
            username: username.clone(),
            amount: -2,
        };

        let url = env::var("STABLE_DIFFUSION_IMG_URL")?;
        let filename = "".to_string();
        let unique_identifier = "".to_string();
        let image_data = download_stable_diffusion_img2img(
            req.prompt.clone(),
            unique_identifier,
            None,
            models::RequestType::Img2ImgFile(filename),
        )
        .await?;
        Ok(())

        // This needs to be moved back to libs
        // let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        // let unique_identifier = format!("{}_{}", timestamp, username);
        // let _ = process_stable_diffusion(
        //     unique_identifier,
        //     image_data,
        //     None,
        //     false,
        // )
        // .await;
        // Ok(())
    }
}
