use anyhow::{self, Result};
use std::fs::File;
use std::io::Read;
use std::process::Command;

pub mod image_generation;

// TODO: We need to create this resized directory, when it doesn't exist
// and make it it not absolute
pub fn resize_image(
    unique_identifier: String,
    filename: String,
) -> Result<(String, Vec<u8>)> {
    let output_path =
        format!("./tmp/screenshots/resized/{}", unique_identifier);
    Command::new("convert")
        .args(&[
            filename,
            "-resize".to_string(),
            "1280x720".to_string(),
            output_path.clone(),
        ])
        .status()?;
    let mut file = File::open(output_path.clone())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok((output_path, buffer))
}
