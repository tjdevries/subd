use serde::Deserialize;

// We aren't using these
// we are parsing just to JSON

#[derive(Deserialize)]
struct FalImage {
    url: String,
    _width: Option<u32>,
    _height: Option<u32>,
    _content_type: Option<String>,
}

#[derive(Deserialize)]
struct FalData {
    images: Vec<FalImage>,
}
