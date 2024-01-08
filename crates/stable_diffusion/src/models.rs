use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GenerateAndArchiveRequest {
    pub prompt: String,
    pub unique_identifier: String,
    pub request_type: RequestType,
    pub set_as_obs_bg: bool,
    pub additional_archive_dir: Option<String>,
    pub strength: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StableDiffusionRequest {
    pub prompt: String,
    pub username: String,
    pub amount: i32,
    pub easing_function_index: i32,
    pub easing_type_index: i32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum RequestType {
    #[default]
    Prompt2Img,
    Img2ImgFile(String),
    Img2ImgURL(String),
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
