use anyhow::{anyhow, Result};
use chrono::Utc;
use instruct_macros::InstructMacro;
use instruct_macros_types::{Parameter, ParameterInfo, StructInfo};
use instructor_ai::from_openai;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::{
    chat_completion::{self, ChatCompletionRequest},
    common::GPT4_O,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn backup_file(src: &str, backup_dir: &str) -> Result<String> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let filename = Path::new(src)
        .file_name()
        .ok_or_else(|| anyhow!("Invalid source file path"))?
        .to_string_lossy();
    let backup_filename = format!("{}/{}_{}", backup_dir, timestamp, filename);
    fs::create_dir_all(backup_dir)
        .map_err(|e| anyhow!("Failed to create backup directory: {}", e))?;
    fs::copy(src, &backup_filename)
        .map_err(|e| anyhow!("Failed to backup the file: {}", e))?;
    Ok(backup_filename)
}

pub fn save_content_to_file(content: &str, dest: &str) -> Result<()> {
    let mut file = File::create(dest)
        .map_err(|e| anyhow!("Failed to create file {}: {}", dest, e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file {}: {}", dest, e))
}
