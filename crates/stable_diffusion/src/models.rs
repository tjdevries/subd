use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct GenerateAndArchiveRequest {
    prompt: String,
    unique_identifier: String,
    request_type: RequestType,
    set_as_obs_bg: bool,
    additional_archive_dir: Option<String>,
    strength: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum RequestType {
    Img2ImgFile(String),
    Img2ImgURL(String),
    
    #[default]
    Prompt2Img,
}

// impl Default for RequestType {
//     fn default() -> Self { RequestType::Prompt2Img }
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct SDResponse {
    pub data: Vec<SDResponseData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SDResponseData {
    pub b64_json: String,
    pub revised_prompt: String,
}

