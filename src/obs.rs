use anyhow::Result;
use obws;
use obws::Client as OBSClient;

// TODO: We need to audit the name all of these

// Scenes
pub const CHARACTERS_SCENE: &str = "Characters";
pub const DEFAULT_SCENE: &str = "Primary";
pub const MEME_SCENE: &str = "memes";

// Sources
pub const DEFAULT_SOURCE: &str = "begin";
pub const UBERDUCK_LOADING_SOURCE: &str = "loading_duck";

// Characters
pub const DEFAULT_STREAM_CHARACTER_SOURCE: &str = "Seal";
pub const TWITCH_STAFF_OBS_SOURCE: &str = "half-life-scientist";

// Voices
pub const TWITCH_STAFF_VOICE: &str = "Randall";
pub const TWITCH_MOD_DEFAULT_VOICE: &str = "brock-samson";
pub const TWITCH_DEFAULT_VOICE: &str = "arbys";

// Dynamic Source
pub const SOUNDBOARD_TEXT_SOURCE_NAME: &str = "Soundboard-Text";

// Dynamic Default Filters
pub const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
pub const DEFAULT_SCROLL_FILTER_NAME: &str = "Default_Scroll";
pub const DEFAULT_SDF_EFFECTS_FILTER_NAME: &str = "Default_SDF_Effects";
pub const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";

// Dynamic Filters
pub const MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
pub const MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";

// Dynamic Filters but Default Filters
pub const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D Transform";
pub const SDF_EFFECTS_FILTER_NAME: &str = "Outline";

// Filter Constant
pub const SINGLE_SETTING_VALUE_TYPE: u32 = 0;
pub const STREAM_FX_INTERNAL_FILTER_NAME: &str = "streamfx-filter-transform";
pub const MOVE_VALUE_INTERNAL_FILTER_NAME: &str = "move_value_filter";
pub const BLUR_FILTER_NAME: &str = "Blur";

pub async fn create_obs_client() -> Result<OBSClient, obws::Error> {
    let obs_websocket_port = subd_types::consts::get_obs_websocket_port()
        .parse::<u16>()
        .unwrap();
    let obs_websocket_address = subd_types::consts::get_obs_websocket_address();
    OBSClient::connect(
        obs_websocket_address.clone(),
        obs_websocket_port,
        Some(""),
    )
    .await
}

// TODO: Find the proper home for this
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
