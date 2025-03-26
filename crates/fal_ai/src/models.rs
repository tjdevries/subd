use serde::Deserialize;

#[derive(Deserialize)]
pub struct FalImage {
    pub url: String,
    pub _width: Option<u32>,
    pub _height: Option<u32>,
    pub _content_type: Option<String>,
}

#[derive(Deserialize)]
pub struct FalData {
    pub images: Vec<FalImage>,
}
