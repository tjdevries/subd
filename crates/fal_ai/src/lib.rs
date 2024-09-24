use anyhow::Result;

pub mod fal_service;
pub mod models;
pub mod utils;

/// Creates an image for a music video using the specified id, prompt, and index.
pub async fn create_image_for_music_video(
    id: &str,
    prompt: &str,
    index: usize,
) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-sdxl";
    let save_dir = format!("./tmp/music_videos/{}/", id);
    fal_service
        .create_image_for_music_video(
            model,
            prompt,
            "landscape_16_9",
            &save_dir,
            index,
        )
        .await
}

/// Creates a turbo image using the "fal-ai/fast-turbo-diffusion" model.
pub async fn create_turbo_image(prompt: &str) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-turbo-diffusion";
    let save_dir = "./tmp/fal_images";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_dir)
        .await
}

/// Creates a fast SD image using the "fal-ai/fast-sdxl" model.
pub async fn create_fast_sd_image(prompt: &str) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/fast-sdxl";
    let save_dir = "./tmp/fal_images";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_dir)
        .await
}

/// Creates an image from a prompt and saves it to the specified folder.
pub async fn create_image_from_prompt_in_folder(
    prompt: &str,
    save_folder: &str,
) -> Result<()> {
    let fal_service = fal_service::FalService::new();
    let model = "fal-ai/stable-cascade";
    fal_service
        .create_image(model, prompt, "landscape_16_9", save_folder)
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
    #[tag(fal)]
    async fn test_create_turbo_image() {
        let prompt = "man raccoon";
        let res = create_turbo_image(prompt).await.unwrap();
        dbg!(res);
        assert!(true);
    }

    // async fn test_create_turbo_image() {
    //     let prompt = "man raccoon";
    //     let res = create_turbo_image(prompt).await.unwrap();
    //     dbg!(res);
    //     assert!(true);
    // }
}
