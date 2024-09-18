use std::env;

pub struct Config {
    pub fal_videos_dir: String,
    pub fal_images_dir: String,
    pub fal_responses_dir: String,
    pub dalle_image_path: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            fal_videos_dir: env::var("FAL_VIDEOS_DIR")
                .unwrap_or_else(|_| "./tmp/fal_videos".to_string()),
            fal_images_dir: env::var("FAL_IMAGES_DIR")
                .unwrap_or_else(|_| "./tmp/fal_images".to_string()),
            fal_responses_dir: env::var("FAL_RESPONSES_DIR")
                .unwrap_or_else(|_| "./tmp/fal_responses".to_string()),
            dalle_image_path: env::var("DALLE_IMAGE_PATH")
                .unwrap_or_else(|_| "./tmp/dalle-1.png".to_string()),
        }
    }
}
