use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScenes {
    pub scenes: Vec<AIScene>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AIScene {
    pub reward_title: String,
    pub base_prompt: String,
    pub base_dalle_prompt: String,
    pub voice: String,
    pub music_bg: String,
    pub cost: usize,
    pub id: Option<Uuid>,
}

#[derive(Deserialize, Debug)]
pub struct ElevenlabsVoice {
    pub voice_id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct VoiceList {
    pub voices: Vec<ElevenlabsVoice>,
}

// Should they be optional???
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamCharacter {
    // text_source: String,
    pub voice: Option<String>,
    pub source: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Voice {
    pub category: String,
    pub display_name: String,
    pub model_id: String,
    pub name: String,
}
