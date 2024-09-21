//// TODO: We need to audit the name all of these
//
//// Dynamic Default Filters
//pub const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
//
//// What's the difference between const and static
//pub const MOVE_SOURCE_FILTER_KIND: &str = "move_source_filter";
//// const MOVE_VALUE_FILTER_KIND: &str = "move_value_filter";
//
//// Scenes
//pub const CHARACTERS_SCENE: &str = "Characters";
//pub const DEFAULT_SCENE: &str = "Primary";
//pub const MEME_SCENE: &str = "memes";
//
//// Sources
//pub const PRIMARY_CAM_SCENE: &str = "Begin";
//pub const DEFAULT_SOURCE: &str = "begin";
//
//// Characters
//pub const DEFAULT_STREAM_CHARACTER_SOURCE: &str = "Seal";
//pub const TWITCH_STAFF_OBS_SOURCE: &str = "Randall";
//
//// Voices
//// pub const TWITCH_STAFF_VOICE: &str = "half-life-scientist";
//pub const TWITCH_STAFF_VOICE: &str = "meowth";
//pub const TWITCH_MOD_DEFAULT_VOICE: &str = "brock-samson";
//// pub const TWITCH_HELPER_VOICE: &str = "e40";
//pub const TWITCH_HELPER_VOICE: &str = "c-3po";
//// pub const TWITCH_HELPER_VOICE: &str = "snoop-dogg";
//// pub const TWITCH_DEFAULT_VOICE: &str = "arbys";
//pub const TWITCH_DEFAULT_VOICE: &str = "neo";
//
//// Dynamic Source
//pub const SOUNDBOARD_TEXT_SOURCE_NAME: &str = "Soundboard-Text";
//
//// Dynamic Default Filters pub const DEFAULT_STREAM_FX_FILTER_NAME: &str = "Default_Stream_FX";
//pub const DEFAULT_SCROLL_FILTER_NAME: &str = "Default_Scroll";
//pub const DEFAULT_SDF_EFFECTS_FILTER_NAME: &str = "Default_SDF_Effects";
//pub const DEFAULT_BLUR_FILTER_NAME: &str = "Default_Blur";
//
//// Dynamic Filters
//pub const MOVE_SCROLL_FILTER_NAME: &str = "Move_Scroll";
//pub const MOVE_BLUR_FILTER_NAME: &str = "Move_Blur";
//pub const MOVE_OUTLINE_FILTER_NAME: &str = "Move_Outline";
//pub const MOVE_STREAM_FX_FILTER_NAME: &str = "Move_Stream_FX";
//
//pub const THREE_D_TRANSITION_PERSPECTIVE_FILTER_NAME: &str =
//    "3D-Transform-Perspective";
//
//// Dynamic Filters but Default Filters
//pub const THE_3D_TRANSFORM_FILTER_NAME: &str = "3D-Transform";
//pub const BLUR_FILTER_NAME: &str = "Blur";
//pub const SCROLL_FILTER_NAME: &str = "Scroll";
//
//// Filter Constant
//pub const BLUR_INTERNAL_FILTER_NAME: &str = "streamfx-filter-blur";
//pub const SCROLL_INTERNAL_FILTER_NAME: &str = "scroll_filter";
//pub const STREAM_FX_INTERNAL_FILTER_NAME: &str = "streamfx-filter-transform";
//pub const MOVE_VALUE_INTERNAL_FILTER_NAME: &str = "move_value_filter";
//pub const SDF_EFFECTS_INTERNAL_FILTER_NAME: &str =
//    "streamfx-filter-sdf-effects";
//
//pub const SINGLE_SETTING_VALUE_TYPE: u32 = 0;
//pub const MULTIPLE_SETTING_VALUE_TYPE: u32 = 1;

pub struct NewVoiceScene {
    pub voice: &'static str,
    pub music: &'static str,
    pub playlist_folder: Option<&'static str>,
}

pub const VOICE_TO_MUSIC: &[(&str, NewVoiceScene)] = &[
    // ("!yoga", NewVoiceScene{ voice: "Thomas", music: "Yoga-BG-Music", playlist_folder: Some("Yoga")}),
    (
        "!begin",
        NewVoiceScene {
            voice: "beginbot",
            music: "Carti-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!horror",
        NewVoiceScene {
            voice: "josh",
            music: "Horror",
            playlist_folder: Some("Horror"),
        },
    ),
    (
        "!comedy",
        NewVoiceScene {
            voice: "Fin",
            music: "Comedy",
            playlist_folder: Some("Comedy"),
        },
    ),
    (
        "!bond",
        NewVoiceScene {
            voice: "james",
            music: "Bond-BG-Music",
            playlist_folder: None,
        },
    ),
    // ("!streamer", NewVoiceScene{ voice: "melkey", music: "Lofi-HipHop-BG-Music", playlist_folder: Some("LofiHipHop")}),
    (
        "!streamer",
        NewVoiceScene {
            voice: "pokimane",
            music: "Lofi-HipHop-BG-Music",
            playlist_folder: Some("LofiHipHop"),
        },
    ),
    (
        "!evil",
        NewVoiceScene {
            voice: "satan",
            music: "Evil-BG-Music",
            playlist_folder: Some("Evil"),
        },
    ),
    (
        "!good",
        NewVoiceScene {
            voice: "god",
            music: "Yoga-BG-Music",
            playlist_folder: Some("Yoga"),
        },
    ),
    (
        "!devito",
        NewVoiceScene {
            voice: "devito",
            music: "IASIP-BG-Music",
            playlist_folder: Some("IASIP"),
        },
    ),
    (
        "!yoga",
        NewVoiceScene {
            voice: "god",
            music: "Yoga-BG-Music",
            playlist_folder: Some("Yoga"),
        },
    ),
    (
        "!wes",
        NewVoiceScene {
            voice: "jeff",
            music: "Wes-BG-Music",
            playlist_folder: Some("Wes"),
        },
    ),
    (
        "!drama",
        NewVoiceScene {
            voice: "Ethan",
            music: "Dramatic-BG-Music",
            playlist_folder: Some("Drama"),
        },
    ),
    (
        "!greed",
        NewVoiceScene {
            voice: "Michael",
            music: "Greed-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!ken",
        NewVoiceScene {
            voice: "James",
            music: "KenBurns-BG-Music",
            playlist_folder: Some("KenBurns"),
        },
    ),
    // ("!hospital", NewVoiceScene{ voice: "prime", music: "Hospital-BG-Music", playlist_folder: Some("Hospital")}),
    (
        "!hospital",
        NewVoiceScene {
            voice: "Bella",
            music: "Hospital-BG-Music",
            playlist_folder: Some("Hospital"),
        },
    ),
    (
        "!sigma",
        NewVoiceScene {
            voice: "Ethan",
            music: "Sigma-BG-Music",
            playlist_folder: Some("Sigma"),
        },
    ),
    (
        "!news",
        NewVoiceScene {
            voice: "Ethan",
            music: "News-1-BG-Music",
            playlist_folder: Some("News"),
        },
    ),
    (
        "!sexy",
        NewVoiceScene {
            voice: "Charlotte",
            music: "Sexy-2-BG-Music",
            playlist_folder: Some("Sexy"),
        },
    ),
    (
        "!romcom",
        NewVoiceScene {
            voice: "Fin",
            music: "Romcom-BG-Music",
            playlist_folder: Some("RomCom"),
        },
    ),
    (
        "!chef",
        NewVoiceScene {
            voice: "Giovanni",
            music: "Chef-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!carti",
        NewVoiceScene {
            voice: "Random",
            music: "Carti-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!nerds",
        NewVoiceScene {
            voice: "Fin",
            music: "Nerd-BG-Music",
            playlist_folder: None,
        },
    ),
    (
        "!earth",
        NewVoiceScene {
            voice: "atten",
            music: "Planet-Earth-BG-Music-1",
            playlist_folder: Some("PlanetEarth"),
        },
    ),
];
