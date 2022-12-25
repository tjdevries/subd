use anyhow::Result;
use obws;
use obws::Client as OBSClient;

// TODO: We need to audit all of these
pub const DEFAULT_SCENE: &str = "Primary";
pub const MEME_SCENE: &str = "memes";

pub const SINGLE_SETTING_VALUE_TYPE: u32 = 0;

pub const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
pub const DEFAULT_SCROLL_FILTER_NAME: &str = "Default_Scroll";
pub const DEFAULT_SDF_EFFECTS_FILTER_NAME: &str = "Default_SDF_Effects";
pub const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";

pub const MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
pub const MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";
pub const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D Transform";
pub const SDF_EFFECTS_FILTER_NAME: &str = "Outline";

// ========== //
// Fetch Info //
// ========== //

pub async fn print_filter_info(
    source: &str,
    words: &str,
    obs_client: &OBSClient,
) -> Result<String> {
    println!("Finding Filter Details {:?}", words);

    let filter_details = match obs_client.filters().get(source, words).await {
        Ok(details) => details,
        Err(_) => {
            println!("Error Fetching Filter Details: {:?}", words);
            return Ok("".to_string());
        }
    };

    println!("Filter Details {:?}", filter_details);
    Ok(String::from(format!("{:?}", filter_details)))
}
