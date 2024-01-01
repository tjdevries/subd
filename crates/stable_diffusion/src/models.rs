use serde::{Deserialize, Serialize};

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

pub enum StableDiffusionRequests {
    StableDiffusionImg2ImgRequest,
    StableDiffusionRequest,
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};
    use std::fs::{self, File};
    use std::io::{Read, Write};

    #[test]
    fn test_parsing_carls_images() {
        let filepath = "./archive/b.json";
        // let srcdir = PathBuf::from(filepath);
        let f = fs::canonicalize(filepath).unwrap();

        let mut file = File::open(f).unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let res: SDResponse = serde_json::from_str(&contents).unwrap();
        let base64 = &res.data[0].b64_json;
        let bytes = general_purpose::STANDARD.decode(base64).unwrap();

        // We actually don't want to be running this, cuz it actually runs and saves
        // We need a good name for this
        // let mut file =
        //     File::create("durf2.png").expect("Failed to create file");
        // file.write_all(&bytes).expect("Failed to write to file");
        //
        // // Unless it's none
        // let _content = &res.choices[0].message.content;

        // assert_eq!(srcdir.to_string(), "".to_string());
    }
}
