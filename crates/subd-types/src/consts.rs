use once_cell::sync::OnceCell;
use twitch_oauth2::{AccessToken, RefreshToken};

/// twitch_bot_oauth is the authentication for the bot that will respond to messages in chat and
/// whispers (TODO). It can possibly be your account, but in general that will be pretty confusing
/// so you should probably use a new and separate bot account
pub fn get_twitch_bot_oauth() -> String {
    static TWITCH_BOT_OAUTH: OnceCell<String> = OnceCell::new();
    TWITCH_BOT_OAUTH
        .get_or_init(|| {
            dotenv::var("SUBD_TWITCH_BOT_OAUTH")
                .expect("SUBD_TWITCH_BOT_OAUTH must be present in .env file")
                .replace("oauth:", "")
                .to_string()
        })
        .clone()
}

/// Prefer get_twitch_broadcaster_oauth() over this functions. Since you can just leak
/// strings so easily (and we don't want that). Unfortunately, pubsub seems to think
/// that we should pass strings directly? As far as I can tell.
pub fn get_twitch_broadcaster_raw() -> String {
    static TWITCH_BROADCASTER_OAUTH: OnceCell<String> = OnceCell::new();
    TWITCH_BROADCASTER_OAUTH
        .get_or_init(|| {
            dotenv::var("SUBD_TWITCH_BROADCASTER_OAUTH")
                // dotenv::var("TWITCH_OAUTH_TOKEN")
                // dotenv::var("BEGINBOT_TWITCH_OAUTH_TOKEN")
                .expect("$SUBD_TWITCH_BROADCASTER_OAUTH must be set")
                .replace("oauth:", "")
                .to_string()
        })
        .clone()
}

pub fn get_twitch_broadcaster_oauth() -> AccessToken {
    static TWITCH_BROADCASTER_OAUTH: OnceCell<AccessToken> = OnceCell::new();
    TWITCH_BROADCASTER_OAUTH
        .get_or_init(|| {
            AccessToken::new(
                dotenv::var("SUBD_TWITCH_BROADCASTER_OAUTH")
                    .expect("$SUBD_TWITCH_BROADCASTER_OAUTH must be set")
                    .replace("oauth:", "")
                    .to_string(),
            )
        })
        .clone()
}

pub fn get_twitch_broadcaster_refresh() -> Option<RefreshToken> {
    static TWITCH_BROADCASTER_REFRESH: OnceCell<Option<RefreshToken>> =
        OnceCell::new();
    TWITCH_BROADCASTER_REFRESH
        .get_or_init(|| dotenv::var("SUBD_TWITCH_BOT_REFRESH").ok().map(RefreshToken::new))
        .clone()
}

/// Get the broadcaster's github token.
///
/// Will return "token <TOKEN>". If the env variable has "token " to start with, it will not
/// duplicate the "token token <TOKEN>" but instead still just return "token <TOKEN>"
pub fn get_github_broadcaster_token() -> String {
    // TODO: Lazy
    // TODO: Should return an option probably and just quit from github functions if you don't have
    // it. As many tokens as possible should not be required (will make things a lot easier later
    // if we have this strategy from the start).
    String::from("token ")
        + &dotenv::var("SUBD_GITHUB_TOKEN")
            .expect("Should have GITHUB_ACCESS token")
            .replace("token ", "")
}

pub fn get_database_url() -> String {
    dotenv::var("DATABASE_URL").expect("DATABASE_URL to exist")
}

pub fn get_twitch_broadcaster_username() -> String {
    dotenv::var("SUBD_TWITCH_BROADCASTER_USERNAME")
        .expect("SUBD_TWITCH_BROADCASTER_USERNAME to exist")
}

pub fn get_twitch_broadcaster_channel_id() -> String {
    dotenv::var("SUBD_TWITCH_BROADCASTER_CHANNEL_ID")
        .expect("SUBD_TWITCH_BROADCASTER_CHANNEL_ID to exist")
}

pub fn get_twitch_bot_username() -> String {
    dotenv::var("SUBD_TWITCH_BOT_USERNAME")
        .expect("SUBD_TWITCH_BOT_USERNAME to exist")
}

pub fn get_twitch_bot_channel_id() -> String {
    dotenv::var("SUBD_TWITCH_BOT_CHANNEL_ID")
        .expect("SUBD_TWITCH_BOT_CHANNEL_ID to exist")
}

pub fn get_obs_websocket_address() -> String {
    dotenv::var("SUBD_OBS_WEBSOCKET_ADDRESS")
        .expect("SUBD_OBS_WEBSOCKET_ADDRESS to exist")
}

pub fn get_obs_websocket_port() -> String {
    dotenv::var("SUBD_OBS_WEBSOCKET_PORT")
        .expect("SUBD_OBS_WEBSOCKET_PORT to exist")
}

pub fn get_obs_test_scene() -> String {
    dotenv::var("SUBD_OBS_TEST_SCENE").expect("SUBD_OBS_TEST_SCENE to exist")
}

pub fn get_obs_test_source() -> String {
    dotenv::var("SUBD_OBS_TEST_SOURCE").expect("SUBD_OBS_TEST_SOURCE to exist")
}

pub fn get_obs_test_filter() -> String {
    dotenv::var("SUBD_OBS_TEST_FILTER").expect("SUBD_OBS_TEST_FILTER to exist")
}

pub fn get_ai_videos_dir() -> String {
    dotenv::var("AI_VIDEOS_DIR")
        .unwrap_or_else(|_| "./tmp/fal_videos".to_string())
}

pub fn get_get_ai_images_dir() -> String {
    dotenv::var("AI_IMAGES_DIR")
        .unwrap_or_else(|_| "./tmp/fal_images".to_string())
}

pub fn get_fal_responses_dir() -> String {
    dotenv::var("FAL_RESPONSES_DIR")
        .unwrap_or_else(|_| "./tmp/fal_responses".to_string())
}

pub fn get_obs_background_image_path() -> String {
    dotenv::var("OBS_BACKGROUND_IMAGE_PATH")
        .unwrap_or_else(|_| "./tmp/dalle-1.png".to_string())
}

pub fn get_ai_twin_obs_source() -> String {
    dotenv::var("AI_TWIN_SOURCE").unwrap_or_else(|_| "bogan".to_string())
}

pub fn get_sdf_effects_filter_name() -> String {
    dotenv::var("SDF_FILTER_EFFECTS_NAME")
        .unwrap_or_else(|_| "Outline".to_string())
}

// I don't think this name is actually static and NOT configurable
pub fn get_sdf_effects_internal_filter_name() -> String {
    dotenv::var("SDF_EFFECTS_INTERNAL_FILTER_NAME")
        .unwrap_or_else(|_| "streamfx-filter-sdf-effects".to_string())
}

pub fn get_move_outline_filter_name() -> String {
    dotenv::var("MOVE_OUTLINE_FILTER_NAME")
        .unwrap_or_else(|_| "Move_Outline".to_string())
}

pub fn get_default_obs_source() -> String {
    dotenv::var("DEFAULT_OBS_SOURCE").unwrap_or_else(|_| "begin".to_string())
}

pub fn get_default_obs_scene() -> String {
    dotenv::var("DEFAULT_OBS_SCENE").unwrap_or_else(|_| "Begin".to_string())
}

pub fn get_primary_camera_scene() -> String {
    dotenv::var("PRIMARY_CAMERA_SCENE").unwrap_or_else(|_| "Begin".to_string())
}

pub fn get_move_internal_filter_name() -> String {
    dotenv::var("MOVE_VALUE_INTERNAL_FILTER_NAME")
        .unwrap_or_else(|_| "move_filter_name".to_string())
}

pub fn get_move_blur_filter_name() -> String {
    dotenv::var("MOVE_BLUR_FILTER_NAME")
        .unwrap_or_else(|_| "Move_Blur".to_string())
}

pub fn get_blur_filter_name() -> String {
    dotenv::var("BLUR_FILTER_NAME").unwrap_or_else(|_| "Blur".to_string())
}

pub fn get_blur_internal_filter_name() -> String {
    dotenv::var("BLUR_INTERNAL_FILTER_NAME")
        .unwrap_or_else(|_| "streamfx-filter-blur".to_string())
}

pub fn get_move_scroll_filter_name() -> String {
    dotenv::var("MOVE_SCROLL_FILTER_NAME")
        .unwrap_or_else(|_| "Move_Scroll".to_string())
}

pub fn get_scroll_filter_name() -> String {
    dotenv::var("SCROLL_FILTER_NAME").unwrap_or_else(|_| "Scroll".to_string())
}

pub fn get_scroll_internal_filter_name() -> String {
    dotenv::var("SCROLL_INTERNAL_FILTER_NAME")
        .unwrap_or_else(|_| "scroll_filter".to_string())
}

pub fn get_stream_fx_internal_filter_name() -> String {
    dotenv::var("STREAM_FX_INTERNAL_FILTER_NAME")
        .unwrap_or_else(|_| "streamfx-filter-transform".to_string())
}

pub fn get_move_stream_fx_filter_name() -> String {
    dotenv::var("MOVE_STREAM_FX_FILTER_NAME")
        .unwrap_or_else(|_| "Move_Stream_FX".to_string())
}

pub fn get_3d_transform_filter_name() -> String {
    dotenv::var("THE_3D_TRANSFORM_FILTER_NAME")
        .unwrap_or_else(|_| "3D-Transform".to_string())
}

pub fn get_meme_scene() -> String {
    dotenv::var("MEME_SCENE").unwrap_or_else(|_| "memes".to_string())
}

pub fn get_default_stream_character_source() -> String {
    dotenv::var("DEFAULT_STREAM_CHARACTER_SOURCE")
        .unwrap_or_else(|_| "Seal".to_string())
}

pub fn get_twitch_default_source() -> String {
    dotenv::var("TWITCH_DEFAULT_SOURCE").unwrap_or_else(|_| "arbys".to_string())
}

pub fn get_twitch_mod_default_voice() -> String {
    dotenv::var("TWITCH_MOD_DEFAULT_VOICE")
        .unwrap_or_else(|_| "brock-samson".to_string())
}

pub fn get_twitch_default_voice() -> String {
    dotenv::var("TWITCH_MOD_DEFAULT_VOICE")
        .unwrap_or_else(|_| "ethan".to_string())
}

pub fn get_twitch_staff_obs_source() -> String {
    dotenv::var("TWITCH_STAFF_OBS_SOURCE")
        .unwrap_or_else(|_| "randall".to_string())
}

pub fn get_twitch_staff_voice() -> String {
    dotenv::var("TWITCH_STAFF_VOICE").unwrap_or_else(|_| "meowth".to_string())
}

pub fn get_twitch_helper_voice() -> String {
    dotenv::var("TWITCH_HELPER_VOICE").unwrap_or_else(|_| "e40".to_string())
}

pub fn get_soundboard_text_source_name() -> String {
    dotenv::var("SOUNDBOARD_TEXT_SOURCE_NAME")
        .unwrap_or_else(|_| "Soundboard-Text".to_string())
}

pub fn get_move_source_filter_kind() -> String {
    dotenv::var("MOVE_SOURCE_FILTER_KIND")
        .unwrap_or_else(|_| "move_source_filter".to_string())
}
