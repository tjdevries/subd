use anyhow::Result;

pub mod fal_service;
pub mod models;
pub mod utils;

/// Creates a turbo image using the "fal-ai/fast-turbo-diffusion" model.
pub async fn create_turbo_image(prompt: &str) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-turbo-diffusion";
    let save_dir = "./tmp/fal_images";

    fal_service
        .create_image(
            model,
            prompt,
            "landscape_16_9",
            save_dir,
            None,
            Some("./tmp/dalle-1.png"),
        )
        .await
}

/// Creates a fast SD image using the "fal-ai/fast-sdxl" model.
pub async fn create_from_fal_api(prompt: &str) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-sdxl";
    // let model = "fal-ai/stable-cascade";
    let save_dir = "./tmp/fal_images";
    fal_service
        .create_image(
            model,
            prompt,
            "landscape_16_9",
            save_dir,
            None,
            Some("./tmp/dalle-1.png"),
        )
        .await
}

/// Creates an image for a music video using the specified id, prompt, and index.
pub async fn create_image_for_music_video(
    id: &str,
    prompt: &str,
    index: usize,
) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-sdxl";
    let save_dir = format!("./tmp/music_videos/{}/", id);

    // This only needs an index to save the name properly
    fal_service
        .create_image(
            model,
            prompt,
            "landscape_16_9",
            &save_dir,
            Some(index),
            None,
        )
        .await
}

/// Creates a video from the given image file path.
pub async fn create_video_from_image(image_file_path: &str) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let save_dir = subd_types::consts::get_ai_videos_dir();
    fal_service
        .create_video_from_image(image_file_path, &save_dir)
        .await
}

/// Submits a request to the Sadtalker model.
pub async fn fal_submit_sadtalker_request(
    source_image_data_uri: &str,
    driven_audio_data_uri: &str,
) -> Result<String> {
    let fal_service = fal_service::FalService::new();
    fal_service
        .submit_sadtalker_request(source_image_data_uri, driven_audio_data_uri)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_tag::tag;

    #[tokio::test]
    #[ignore]
    #[tag(fal)]
    async fn test_create_turbo_image() {
        let prompt = "man raccoon";
        let res = create_turbo_image(prompt).await.unwrap();
        dbg!(res);
        assert!(true);
    }

    #[tokio::test]
    #[tag(fal)]
    async fn test_create_image_from_fal_api() {
        let prompt = "fancy dog";
        let res = create_from_fal_api(prompt).await.unwrap();
        dbg!(res);
        assert!(true);
    }
}
