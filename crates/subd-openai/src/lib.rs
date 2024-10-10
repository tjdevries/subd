use anyhow::{anyhow, Result};
use chrono::Utc;
use openai::{
    chat::{
        ChatCompletion, ChatCompletionContent, ChatCompletionMessage,
        ChatCompletionMessageRole, ImageUrl, VisionMessage,
    },
    set_key,
};
use openai_api_rs::v1::common::GPT4_O;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fs::File, io::Write};

pub mod ai_styles;
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

pub async fn ask_chat_gpt(
    user_input: &str,
    base_content: &str,
) -> Result<ChatCompletionMessage> {
    set_key(env::var("OPENAI_API_KEY")?);

    let messages = vec![
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(ChatCompletionContent::Message(Some(
                base_content.to_string(),
            ))),
            name: None,
            function_call: None,
        },
        ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(ChatCompletionContent::Message(Some(
                user_input.to_string(),
            ))),
            name: None,
            function_call: None,
        },
    ];

    let chat_completion = ChatCompletion::builder(GPT4_O, messages.clone())
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
