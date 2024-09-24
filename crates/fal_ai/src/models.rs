use serde::Deserialize;

// We aren't using these
// we are parsing just to JSON

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
