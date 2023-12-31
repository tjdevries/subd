use anyhow::anyhow;
use anyhow::Result;
use chrono::Utc;
use core::pin::Pin;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub trait GenerateImage {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>;
}

// pub enum AiImageRequests {
//     DalleRequest,
//     StableDiffusionRequest,
// }

pub fn unique_archive_filepath(
    index: usize,
    username: String,
) -> Result<(PathBuf, String), anyhow::Error> {
    println!("Generating Unique Archive Filepath");
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}_{}", timestamp, index, username);
    let filename = format!("./archive/{}.png", unique_identifier);
    let filepath = Path::new(&filename);
    let pathbuf = PathBuf::from(filepath);
    Ok((pathbuf, unique_identifier))
    // Only do it for the archive
    // Umm This requires the file exists...which wasn't the intent of this
    // fs::canonicalize(pathbuf.clone())
    //     .map_err(|e| anyhow!("{:?} {}", pathbuf, e.to_string()))
    //     .map(|v| (v, unique_identifier))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive() -> Result<()> {
        // unique_ar().map(|(pathbuf, t)| assert_eq!(t, pathbuf.to_str().unwrap()))
        let (pathbuf, t) = unique_archive_filepath(1, "beginbot".to_string())?;
        let p = pathbuf
            .to_str()
            .ok_or(anyhow!("Error converting pathbuf to str"))?
            .to_string();
        assert_eq!(t, p);
        Ok(())
    }
}
