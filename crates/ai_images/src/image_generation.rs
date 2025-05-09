use anyhow::Result;
use chrono::Utc;
use core::pin::Pin;
use std::path::Path;
use std::path::PathBuf;

// TODO: We shouldn't be using warp::Future
pub trait GenerateImage {
    fn generate_image(
        &self,
        prompt: String,
        save_folder: Option<String>,
        set_as_obs_bg: bool,
    ) -> Pin<Box<(dyn warp::Future<Output = String> + std::marker::Send + '_)>>;
}

pub fn unique_archive_filepath(
    index: usize,
    username: String,
) -> Result<(PathBuf, String)> {
    println!("Generating Unique Archive Filepath");
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let unique_identifier = format!("{}_{}_{}", timestamp, index, username);
    let filename = format!("./archive/{}.png", unique_identifier);
    let filepath = Path::new(&filename);
    let pathbuf = PathBuf::from(filepath);
    Ok((pathbuf, unique_identifier))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_archive() -> Result<()> {
        let (pathbuf, _t) = unique_archive_filepath(1, "beginbot".to_string())?;
        let _p = pathbuf
            .to_str()
            .ok_or(anyhow!("Error converting pathbuf to str"))?
            .to_string();
        // assert_eq!(t, p);
        Ok(())
    }
}
