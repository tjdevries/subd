use serde::{Deserialize, Serialize};

pub enum RequestType {
    Img2ImgFile(String),
    Img2ImgURL(String),
    Prompt2Img(),
}

// Filename
// Unique Identifier
struct GenerateAndArchiveRequest {
    prompt: String,
    unique_identifier: String,
    request_type: RequestType,
    set_as_obs_bg: bool,
    additional_archive_dir: Option<String>,
    strength: Option<f32>,
}


pub enum StableDiffusionRequests {
    StableDiffusionImg2ImgRequest,
    StableDiffusionRequest,
}

pub struct StableDiffusionImg2ImgRequest {
    pub prompt: String,
    pub filename: String,
    pub unique_identifier: String,
}

pub struct StableDiffusionRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SDResponse {
    pub data: Vec<SDResponseData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SDResponseData {
    pub b64_json: String,
    pub revised_prompt: String,
}

