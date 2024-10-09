use anyhow::{anyhow, Result};
use chrono::Utc;
use instruct_macros::InstructMacro;
use instruct_macros_types::{Parameter, ParameterInfo, StructInfo};
use instructor_ai::from_openai;
use openai::{
    chat::{
        ChatCompletion, ChatCompletionContent, ChatCompletionMessage,
        ChatCompletionMessageRole, ImageUrl, VisionMessage,
    },
    set_key,
};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::{
    chat_completion::{self, ChatCompletionRequest},
    common::GPT4_O,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

pub mod dalle;

#[derive(Debug, Serialize, Deserialize)]
struct VisionResponse {
    choices: Vec<VisionChoice>,
    id: String,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIUsage {
    completion_tokens: u32,
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisionChoice {
    fininsh_reason: Option<String>,
    index: u8,
    message: VisionChoiceContent,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisionChoiceContent {
    content: String,
    role: String,
}

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
    content: String,
    base_path: Option<&str>,
) -> Result<()> {
    let client = openai_api_rs::v1::api::Client::new(
        env::var("OPENAI_API_KEY").unwrap(),
    );
    let instructor_client = from_openai(client);
    let contents = html_file_contents(base_path)?;

    let prompt = format!(
        "Generate excellent high-quality detailed JS for the provided HTML. Make the page as animated and fun as possible. Use libraries like three.js, phaser.js, 3D.js Summarize the following content and use it to influence the animation and JavaScript. Be Creative.: {} Make it all savable as a styles.js file, for the following HTML: {}",
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

    let js_file = "../../static/styles.js";
    println!("{:?}", result);
    match backup_file(js_file, "./static") {
        Ok(_) => println!("Successfully backed up styles.js file"),
        Err(e) => println!("Failed to backup styles.js file: {}", e),
    };
    save_content_to_file(&result.javascript, js_file)?;
    Ok(())
}

pub async fn generate_ai_css(
    content: String,
    base_path: Option<&str>,
) -> Result<()> {
    let client = Client::new(
        env::var("OPENAI_API_KEY")
            .map_err(|e| anyhow::anyhow!("OPENAI_API_KEY not set: {}", e))?,
    );
    let instructor_client = from_openai(client);
    let contents = html_file_contents(base_path)?;

    let content_prompt = format!("Summarize the following content and use it to influence the styling: {}", content);
    let prompt_parts = vec![
        "Generate excellent high-quality detailed CSS for the provided HTML. Use as many CSS properties as possible.",
        "Make the overall style consistent, with a color theme and font-selection that is cohesive but fun.",
        "Use as many CSS properties as possible, for example: animation-duration, animation-delay, animation-direction, animation-fill-mode, animation, mix-blend-mode, backdrop-filter, filter, text-shadow, box-shadow, border-image, mask, background-clip, transform, perspective, isolation, object-fit, object-position, animation, transition, shape-outside, conic-gradient, linear-gradient, radial-gradient, font-variant, text-stroke, aspect-ratio, grid-template-areas, align-self, object-position, word-wrap, resize, appearance, backface-visibility, blend-mode, font-display etc.",
        "Include as many interactive changes as possible, like on:hover. Make all links do something bold and dynamic on hover.",
        "Also feel free to try different layouts using things like display: grid.",
        "Use a wide variety of bold unique fonts, which you need to specify to import.",
        "Use as many animations and transitions as possible.",
        "Make sure you use things like transform: rotate(360deg); Prioritize movement with transform.",
        &content_prompt,
        "Make it all savable as a styles.css file, for the following HTML:",
    ];

    let prompt = format!("{} {}", prompt_parts.join(" "), contents);

    let req = ChatCompletionRequest::new(
        GPT4_O.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result: AIStylesResponse =
        get_chat_completion_with_retries(&instructor_client, req, 3).await?;

    println!("{:?}", result);
    let css_file = "../../static/styles.css";
    match backup_file(css_file, "../../static") {
        Ok(_) => println!("Successfully backed up CSS file"),
        Err(e) => println!("Failed to backup CSS file: {}", e),
    };
    save_content_to_file(&result.css, css_file)?;
    Ok(())
}

pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage> {
    set_key(env::var("OPENAI_API_KEY")?);

    let messages = vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(ChatCompletionContent::Message(Some(base_content))),
            name: None,
            function_call: None,
        },
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(ChatCompletionContent::Message(Some(user_input))),
            name: None,
            function_call: None,
        },
    ];

    let chat_completion = ChatCompletion::builder(&GPT4_O, messages.clone())
        .create()
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Error creating ChatCompletion builder: {}",
                e.to_string()
            )
        })?;

    chat_completion
        .choices
        .first()
        .ok_or_else(|| "Error finding first Chat GPT response".to_string())
        .map(|m| m.message.clone())
        .map_err(|e| anyhow::anyhow!("Error getting Chat GPT response: {}", e))
}

pub async fn ask_gpt_vision2(
    image_path: &str,
    image_url: Option<&str>,
) -> Result<String> {
    let api_key = env::var("OPENAI_API_KEY")?;
    let full_path = match image_url {
        Some(url) => url.to_string(),
        None => {
            let base64_image = subd_image_utils::encode_image(image_path)?;
            format!("data:image/jpeg;base64,{}", base64_image)
        }
    };

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", api_key).parse()?,
    );

    let payload = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "What’s in this image?"
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": full_path
                        }
                    }
                ]
            }
        ],
        "max_tokens": 300
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&payload)
        .send()
        .await?;

    let response_json: Value = response.json().await?;

    let filename = format!("{}.json", Utc::now().timestamp());
    let filepath =
        format!("/home/begin/code/subd/tmp/Archive/vision/{}", filename);
    let mut file = File::create(filepath)?;
    file.write_all(response_json.to_string().as_bytes())?;

    let vision_res: VisionResponse = serde_json::from_value(response_json)?;
    let content = &vision_res.choices[0].message.content;
    Ok(content.to_string())
}

pub async fn ask_gpt_vision(
    user_input: String,
    image_url: String,
) -> Result<ChatCompletionMessage> {
    set_key(env::var("OPENAI_API_KEY")?);

    let new_content = ChatCompletionContent::VisionMessage(vec![
        VisionMessage::Text {
            content_type: "text".to_string(),
            text: user_input,
        },
        VisionMessage::Image {
            content_type: "image_url".to_string(),
            image_url: ImageUrl { url: image_url },
        },
    ]);

    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(new_content),
        name: None,
        function_call: None,
    }];

    let chat_completion =
        ChatCompletion::builder("gpt-4-vision-preview", messages.clone())
            .create()
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    chat_completion
        .choices
        .first()
        .ok_or_else(|| "Error finding GPT Vision first response".to_string())
        .map(|m| m.message.clone())
        .map_err(|e| anyhow::anyhow!("Error w/ GPT Vision: {}", e))
}

fn html_file_contents(base_path: Option<&str>) -> Result<String> {
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

    #[tokio::test]
    async fn test_generating_css() {
        let res =
            generate_ai_css("Neon".to_string(), Some("../../templates")).await;
        assert!(res.is_ok(), "{:?}", res);
        let res =
            generate_ai_js("Neon".to_string(), Some("../../templates")).await;
        assert!(res.is_ok(), "{:?}", res);
    }
}
