use anyhow::{anyhow, Result};
use instruct_macros::InstructMacro;
use instruct_macros_types::Parameter;
use instruct_macros_types::{ParameterInfo, StructInfo};
use instructor_ai::from_openai;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{self, ChatCompletionRequest},
    common::GPT3_5_TURBO,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
pub struct MusicVideoScenes {
    pub scenes: Vec<MusicVideoScene>,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
pub struct MusicVideoScene {
    pub image_prompt: String,
    pub camera_move: String,
    pub image_name: Option<String>,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
pub struct RunwayPrompt {
    pub scene_description: String,
    pub camera_move: String,
}

// ==============================================================

pub async fn generate_scene_from_prompt(
    prompt: String,
) -> Result<RunwayPrompt> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result = match instructor_client.chat_completion::<RunwayPrompt>(req, 3)
    {
        Ok(scenes) => scenes,
        Err(e) => {
            eprintln!("Error generating Runway Prompts: {:?}", e);
            return Err(anyhow!("Failed to generate Runway prompts"));
        }
    };

    println!("{:?}", result);
    Ok(result)
}

pub async fn generate_scene_prompt(
    lyrics: String,
    title: String,
) -> Result<MusicVideoScene> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let prompt = format!(
        "Describe 1 scene in vivid detail for a Music Video: image_prompt and camera_move based on the following Lyrics: {} and Title: {}. They should be fun scenes that stick to an overall theme based on the title.",
        lyrics, title);

    println!("\tUsing the Prompt: {}", prompt);

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result =
        match instructor_client.chat_completion::<MusicVideoScene>(req, 3) {
            Ok(scenes) => scenes,
            Err(e) => {
                eprintln!("Error generating scene prompts: {:?}", e);
                return Err(anyhow!("Failed to generate scene prompts"));
            }
        };

    println!("{:?}", result);
    Ok(result)
}
pub async fn generate_scene_prompts(
    lyrics: String,
    title: String,
) -> Result<MusicVideoScenes> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let prompt = format!(
        "Describe 5 scenes for a Music Video: image_prompt and camera_move based on the following Lyrics: {} and Title: {}. They should be fun scenes that stick to an overall theme based on the title.",
        lyrics, title);

    println!("\tUsing the Prompt: {}", prompt);

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result =
        match instructor_client.chat_completion::<MusicVideoScenes>(req, 3) {
            Ok(scenes) => scenes,
            Err(e) => {
                eprintln!("Error generating scene prompts: {:?}", e);
                return Err(anyhow!("Failed to generate scene prompts"));
            }
        };

    println!("{:?}", result);
    Ok(result)
}
