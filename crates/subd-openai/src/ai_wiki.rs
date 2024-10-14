use crate::utils;
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
struct AIWikiResponse {
    early_life: String,
    education: String,
    controversy: String,
    pop_culture_references: String,
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
                        "Failed after {} attempts",
                        max_attempts
                    ));
                }
            }
        }
    }
    unreachable!()
}

pub async fn generate_ai_wiki(
    username: String,
    destination: &str,
    content: String,
    html_to_animate_folder: Option<&str>,
) -> Result<()> {
    let client = create_openai_client()?;
    let instructor_client = from_openai(client);
    let prompt = create_prompt(&content);
    let req = create_chat_completion_request(&prompt);

    let result: AIWikiResponse =
        get_chat_completion_with_retries(&instructor_client, req, 3).await?;

    println!("{:?}", result);
    backup_and_save(&username, destination, &result)?;
    Ok(())
}

fn create_openai_client() -> Result<Client> {
    let api_key = env::var("OPENAI_API_KEY").map_err(|e| {
        eprintln!("Error retrieving OPENAI_API_KEY: {}", e);
        anyhow!("Failed to retrieve OPENAI_API_KEY: {}", e)
    })?;
    Ok(Client::new(api_key))
}

fn create_prompt(content: &str) -> String {
    format!(
        "Generate excellent high-quality detailed JS for the provided HTML. Make the page as animated and fun as possible. Use libraries like three.js, D3.js, mermaid.js. Create some Charts with mermaid.js | Print whats happening to console.log as often as possible. Summarize the following content and use it to influence the animation and JavaScript and be creative : {}. Make it all savable as a styles.js file, for the following HTML: ",
        content
    )
}

fn create_chat_completion_request(prompt: &str) -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        GPT4_O.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt.to_string()),
            name: None,
        }],
    )
}

fn backup_and_save(
    username: &str,
    destination: &str,
    result: &AIWikiResponse,
) -> Result<()> {
    match utils::backup_file(destination, &format!("./archive/{}", username)) {
        Ok(_) => println!("Successfully backed file"),
        Err(e) => println!("Failed to backup file: {}", e),
    };

    let html = format!(
        "<h2>Early Life</h2>\n{}\n<h2>Education</h2>\n{}\n<h2>Controversy</h2>\n{}\n<h2>Pop Culture References</h2>\n{}",
        result.early_life, result.education, result.controversy, result.pop_culture_references
    );
    utils::save_content_to_file(&html, destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_wiki() {
        let username = "carlvandergeest";
        // let username = "zanuss";
        let wiki_file = "../../static/wiki.html";
        let res = generate_ai_wiki(
            username.to_string(),
            wiki_file,
            format!("Generate a Wikipedia article based on chat history for the user: {}", username.to_string()),
            Some("../../templates"),
        )
        .await;
        assert!(res.is_ok(), "{:?}", res);
    }
}
