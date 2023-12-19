use crate::dalle::GenerateImage;
use crate::openai;
use anyhow::Result;
use obws::Client as OBSClient;
use rodio::*;


// TODO: I don't like the name
pub async fn create_screenshot_variation(
    sink: &Sink,
    obs_client: &OBSClient,
    filename: String,
    ai_image_req: &impl GenerateImage,
    prompt: String,
    source: String,
) -> Result<String, String> {
    // let _ = audio::play_sound(&sink).await;

    let _ = openai::save_screenshot(&obs_client, &source, &filename).await;

    let description = openai::ask_gpt_vision2(&filename, None)
        .await
        .map_err(|e| e.to_string())?;

    let new_description = format!(
        "{} {} . The most important thing to focus on is: {}",
        prompt, description, prompt
    );

    let dalle_path = ai_image_req
        .generate_image(new_description, Some("timelapse".to_string()), false)
        .await;

    if dalle_path == "".to_string() {
        return Err("Dalle Path is empty".to_string());
    }

    println!("Dalle Path: {}", dalle_path);
    Ok(dalle_path)
}
