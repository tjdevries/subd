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

impl GenerateAndArchiveRequest {
    pub fn new(
        prompt: String,
        unique_identifier: String,
        request_type: RequestType,
    ) -> GenerateAndArchiveRequest {
        Self {
            prompt,
            unique_identifier,
            request_type,
            ..Default::default()
        }
    }

    pub fn builder() -> GenerateAndArchiveRequestBuilder {
        GenerateAndArchiveRequestBuilder {
            ..Default::default()
        }
    }
}

impl GenerateAndArchiveRequestBuilder {
    pub fn build(self) -> GenerateAndArchiveRequest {
        GenerateAndArchiveRequest {
            prompt: self.prompt,
            unique_identifier: self.unique_identifier,
            request_type: self.request_type,
            set_as_obs_bg: self.set_as_obs_bg.unwrap_or(false),
            additional_archive_dir: self.additional_archive_dir.clone(),
            strength: self.strength,
        }
    }

    pub fn new(
        prompt: String,
        unique_identifier: String,
        request_type: RequestType,
    ) -> GenerateAndArchiveRequestBuilder {
        Self {
            prompt,
            unique_identifier,
            request_type,
            ..Default::default()
        }
    }

    pub fn set_as_obs_bg(mut self, set_as_obs_bg: bool) -> Self {
        self.set_as_obs_bg = Some(set_as_obs_bg);
        self
    }

    pub fn additional_archive_dir(
        mut self,
        additional_archive_dir: String,
    ) -> Self {
        self.additional_archive_dir = Some(additional_archive_dir);
        self
    }

    pub fn strength(mut self, strength: f32) -> Self {
        self.strength = Some(strength);
        self
    }
}

#[derive(Default, Debug)]
pub struct GenerateAndArchiveRequestBuilder {
    pub prompt: String,
    pub unique_identifier: String,
    pub request_type: RequestType,
    pub set_as_obs_bg: Option<bool>,
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
