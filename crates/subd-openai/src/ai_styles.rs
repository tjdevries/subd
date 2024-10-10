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

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct MusicVideoScenes {
    scenes: Vec<MusicVideoScene>,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct MusicVideoScene {
    image_prompt: String,
    camera_move: String,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct AIJavascriptResponse {
    javascript: String,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct AIStylesResponse {
    css: String,
}

async fn get_chat_completion_with_retries<T>(
    instructor_client: &instructor_ai::InstructorClient,
    req: ChatCompletionRequest,
    max_attempts: u32,
) -> Result<T>
where
    T: serde::de::DeserializeOwned + instruct_macros_types::InstructMacro,
{
    for attempt in 1..=max_attempts {
        match instructor_client.chat_completion::<T>(req.clone(), 3) {
            Ok(response) => return Ok(response),
            Err(e) => {
                println!("Attempt {} failed: {}", attempt, e);
                if attempt == max_attempts {
                    return Err(anyhow!(
                        "Failed to get chat completion after {} attempts",
                        max_attempts
                    ));
                }
            }
        }
    }
    unreachable!()
}

pub async fn generate_ai_js(
    song_id: String,
    destination: &str,
    content: String,
    html_to_animate_folder: Option<&str>,
) -> Result<()> {
    let client = openai_api_rs::v1::api::Client::new(
        env::var("OPENAI_API_KEY").map_err(|e| {
            eprintln!("Error retrieving OPENAI_API_KEY: {}", e);
            anyhow::anyhow!("Failed to retrieve OPENAI_API_KEY: {}", e)
        })?,
    );
    let instructor_client = from_openai(client);
    let contents = html_file_contents(html_to_animate_folder)?;

    let prompt = format!(
        "Generate excellent high-quality detailed JS for the provided HTML. Make the page as animated and fun as possible. Use libraries like three.js, D3.js, mermaid.js. Create some Charts with mermaid.js | Print whats happening to console.log as often as possible. Summarize the following content and use it to influence the animation and JavaScript and be creative : {}. Make it all savable as a styles.js file, for the following HTML: {}",
        content, contents
    );

    let req = ChatCompletionRequest::new(
        GPT4_O.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result: AIJavascriptResponse =
        get_chat_completion_with_retries(&instructor_client, req, 3).await?;

    println!("{:?}", result);
    match backup_file(destination, &format!("./archive/{}", song_id)) {
        Ok(_) => println!("Successfully backed up styles.js file"),
        Err(e) => println!("Failed to backup styles.js file: {}", e),
    };
    save_content_to_file(&result.javascript, destination)?;
    Ok(())
}

pub async fn generate_ai_css(
    song_id: String,
    destination: &str,
    content: String,
    html_to_style_folder: Option<&str>,
) -> Result<()> {
    let client = Client::new(
        env::var("OPENAI_API_KEY")
            .map_err(|e| anyhow::anyhow!("OPENAI_API_KEY not set: {}", e))?,
    );
    let instructor_client = from_openai(client);
    let contents = html_file_contents(html_to_style_folder)?;

    let content_prompt = format!("Summarize the following content and use it to influence the styling: {}", content);
    let prompt_parts = ["Generate excellent high-quality detailed CSS for the provided HTML. Use as many CSS properties as possible.",
        "Make the overall style consistent, with a color theme and font-selection that is cohesive but fun.",
        "Use as many CSS properties as possible, for example: animation-duration, animation-delay, animation-direction, animation-fill-mode, animation, mix-blend-mode, backdrop-filter, filter, text-shadow, box-shadow, border-image, mask, background-clip, transform, perspective, isolation, object-fit, object-position, animation, transition, shape-outside, conic-gradient, linear-gradient, radial-gradient, font-variant, text-stroke, aspect-ratio, grid-template-areas, align-self, object-position, word-wrap, resize, appearance, backface-visibility, blend-mode, font-display etc.",
        "Include as many interactive changes as possible, like on:hover. Make all links do something bold and dynamic on hover.",
        "Also feel free to try different layouts using things like display: grid.",
        "Use a wide variety of bold unique fonts, which you need to specify to import.",
        "Use as many animations and transitions as possible.",
        "Make sure you use things like transform: rotate(360deg); Prioritize movement with transform.",
        "include padding around the body.",
        &content_prompt,
        "Make it all savable as a styles.css file, for the following HTML:"];

    let prompt = format!("{} {}", prompt_parts.join(" "), contents);

    let req = ChatCompletionRequest::new(
        GPT4_O.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    // Can we get an ID in here
    let result: AIStylesResponse =
        get_chat_completion_with_retries(&instructor_client, req, 3).await?;

    println!("{:?}", result);

    // Where should we save the CSS
    // This backup destination is wrong
    match backup_file(destination, &format!("./archive/{}", song_id)) {
        Ok(_) => println!("Successfully backed up CSS file"),
        Err(e) => println!("Failed to backup CSS file: {}", e),
    };
    save_content_to_file(&result.css, destination)?;
    Ok(())
}

// TODO: We should take in a list of these html files
pub fn html_file_contents(base_path: Option<&str>) -> Result<String> {
    let base_path = base_path.unwrap_or("./templates");
    let file_names = [
        "base",
        "home",
        "songs",
        "song",
        "users",
        "charts",
        "current_song",
    ];
    let mut contents = String::new();

    for name in &file_names {
        let filepath = format!("{}/{}.html", base_path, name);
        let file_contents = fs::read_to_string(&filepath)
            .map_err(|e| anyhow!("Failed to read {}: {}", filepath, e))?;
        contents.push_str(&file_contents);
        contents.push(' ');
    }
    Ok(contents.trim().to_string())
}

fn backup_file(src: &str, backup_dir: &str) -> Result<String> {
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

fn save_content_to_file(content: &str, dest: &str) -> Result<()> {
    let mut file = File::create(dest)
        .map_err(|e| anyhow!("Failed to create file {}: {}", dest, e))?;
    file.write_all(content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file {}: {}", dest, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // This will backup in the wrong area if run
    #[tokio::test]
    async fn test_generating_css() {
        let css_file = "../../static/styles.css";
        let res = generate_ai_css(
            "FAKE_SONG_ID".to_string(),
            css_file,
            "Neon".to_string(),
            Some("../../templates"),
        )
        .await;
        assert!(res.is_ok(), "{:?}", res);

        let js_file = "../../static/styles.js";
        let res = generate_ai_js(
            "FAKE_SONG_ID".to_string(),
            js_file,
            "Neon".to_string(),
            Some("../../templates"),
        )
        .await;
        assert!(res.is_ok(), "{:?}", res);
    }
}
