// use anyhow::Error;
use crate::dalle;
use crate::dalle::GenerateImage;
use anyhow::Result;
use base64;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use obws::requests::sources::SaveScreenshot;
use obws::Client;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::io::{self, Read};
use std::path::Path;

use openai::{
    chat::{
        ChatCompletion, ChatCompletionContent, ChatCompletionMessage,
        ChatCompletionMessageRole, ImageUrl, VisionMessage,
    },
    set_key,
};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct VisionResponse {
    choices: Vec<VisionChoice>,
    id: String,
    model: String,
    usage: OpenAIUsage,
    //     "created": 1702696712,
    //     "object": "chat.completion",
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

// I want this to exist somewhere else
// probably in a Crate
pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    set_key(env::var("OPENAI_API_KEY").unwrap());

    let base_message = ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(ChatCompletionContent::Message(Some(base_content))),
        name: None,
        function_call: None,
    };

    println!(
        "JSON:\n\n {}",
        serde_json::to_string(&base_message).unwrap()
    );

    let mut messages = vec![base_message];

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(ChatCompletionContent::Message(Some(user_input))),
        name: None,
        function_call: None,
    });

    println!("pre ask_chat_gpt completion");
    // let model = "gpt-4";
    let model = "gpt-3.5-turbo";

    let chat_completion =
        match ChatCompletion::builder(model.clone(), messages.clone())
            .create()
            .await
        {
            Ok(completion) => completion,
            Err(e) => {
                println!("\n\tChat GPT error occurred: {}", e);
                return Err(e);
            }
        };

    let returned_message =
        chat_completion.choices.first().unwrap().message.clone();
    Ok(returned_message)
}

pub async fn ask_gpt_vision2(
    image_path: &str,
    image_url: Option<&str>,
) -> Result<String, anyhow::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let full_path = match image_url {
        Some(url) => url.to_string(),
        None => {
            let base64_image =
                encode_image(image_path).expect("Failed to encode image");
            format!("data:image/jpeg;base64,{}", base64_image)
        }
    };

    let client = reqwest::Client::new();
    let headers = reqwest::header::HeaderMap::from_iter(vec![
        (
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        ),
        (
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        ),
    ]);

    let payload = json!({
        "model": "gpt-4-vision-preview",
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

    let now: DateTime<Utc> = Utc::now();
    let filename = format!("{}.json", now.timestamp());
    let filepath =
        format!("/home/begin/code/subd/tmp/Archive/vision/{}", filename);
    let mut file = File::create(filepath).unwrap();
    file.write_all(response_json.to_string().as_bytes())
        .unwrap();

    //
    let vision_res: VisionResponse =
        match serde_json::from_str(&response_json.to_string()) {
            Ok(res) => res,
            Err(e) => {
                println!("Error parsing JSON: {}", e);
                return Err(e.into());
            }
        };
    let content = &vision_res.choices[0].message.content;
    Ok(content.to_string())
}

pub async fn telephone(
    url: String,
    prompt: String,
    num_connections: u8,
    ai_image_req: &impl GenerateImage,
) -> Result<String, anyhow::Error> {
    let first_description = match ask_gpt_vision2("", Some(&url)).await {
        Ok(description) => description,
        Err(e) => {
            eprintln!("Error asking GPT Vision for description: {}", e);
            return Err(e.into());
        }
    };

    let description = format!("{} {}", first_description, prompt);
    let mut dalle_path =
        ai_image_req.generate_image(description, None, false).await;

    for _ in 0..num_connections {
        let description = ask_gpt_vision2(&dalle_path, None).await.unwrap();
        let prompt = format!("{} {}", description, prompt);
        let req = dalle::DalleRequest {
            prompt: prompt.clone(),
            username: "beginbot".to_string(),
            amount: 1,
        };
        dalle_path = req.generate_image(prompt, None, false).await;
    }
    Ok(dalle_path)
}

// This doesn't go here
pub async fn save_screenshot(
    client: &Client,
    source_name: &str,
    file_path: &str,
) -> Result<()> {
    // save_screenshot(&client, "YourSourceName", "/path/to/save/screenshot.png").await?;
    let p = Path::new(file_path);

    client
        .sources()
        .save_screenshot(SaveScreenshot {
            source: &source_name.to_string(),
            format: "png",
            file_path: p,
            width: None,
            height: None,
            compression_quality: None,
        })
        .await?;

    Ok(())
}

// This doesn't belong here
fn encode_image(image_path: &str) -> io::Result<String> {
    let mut file = File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let b64 = general_purpose::STANDARD.encode(&buffer);
    Ok(b64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telephone() {
        // let first_image = "https://d23.com/app/uploads/2013/04/1180w-600h_mickey-mouse_1.jpg";
        // let first_image = "https://mario.wiki.gallery/images/thumb/1/13/Funky_Kong_Artwork_-_Donkey_Kong_Country_Tropical_Freeze.png/600px-Funky_Kong_Artwork_-_Donkey_Kong_Country_Tropical_Freeze.png";
        // let first_image = "https://static.wikia.nocookie.net/donkeykong/images/7/72/Candy.PNG/revision/latest/scale-to-width-down/110?cb=20130203073312";
        // let first_image = "https://www.tbstat.com/wp/uploads/2023/05/Fvz9hOIXwAEaIR8-669x675.jpeg";
        let _first_image = "https://upload.wikimedia.org/wikipedia/en/thumb/3/3b/SpongeBob_SquarePants_character.svg/1200px-SpongeBob_SquarePants_character.svg.png";

        // let res = telephone(first_image.to_string(), "more chill".to_string(), 10).await.unwrap();
        // let res =
        //     telephone2(first_image.to_string(), "More Memey".to_string(), 10)
        //         .await
        //         .unwrap();
        // assert_eq!("", res);
    }

    #[tokio::test]
    async fn test_gpt_vision() {
        let _user_input = "whats in this image".to_string();
        let _image_url = "https://upload.wikimedia.org/wikipedia/en/7/7d/Donkey_Kong_94_and_64_characters.png".to_string();
        // let res = ask_gpt_vision(user_input, image_url).await;

        let _image_path = "/home/begin/code/BeginGPT/stick_boi.jpg";

        // let res = ask_gpt_vision(user_input, image_url).await.unwrap();
        // dbg!(&res);

        //let res = ask_gpt_vision2(image_path, Some(&image_url)).await.unwrap();

        // // let res = vision_time(image_path, None).await.unwrap();
        // let now: DateTime<Utc> = Utc::now();
        // let filename = format!("{}.json", now.timestamp());
        // let filepath= format!("/home/begin/code/subd/tmp/Archive/vision/{}", filename);
        // let mut file = File::create(filepath).unwrap();
        // file.write_all(res.to_string().as_bytes()).unwrap();
        // let vision_res: VisionResponse = serde_json::from_str(&res.to_string()).unwrap();
        // let content = &vision_res.choices[0].message.content;

        // Why can't I convert this to JSON???
        // println!("\nVision Time: {}", &res);
        // let res = ask_chat_gpt("".to_string(), user_input).await;
        // dbg!(&res);
        // assert_eq!("", res);
    }

    #[tokio::test]
    async fn test_parsing_vision_responses() {
        // let vision_data = File::read(, buf)
        let filepath =
            "/home/begin/code/subd/tmp/Archive/vision/1702696715.json";
        let mut file = File::open(filepath).unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);

        let res: VisionResponse = serde_json::from_str(&contents).unwrap();

        // Unless it's none
        let _content = &res.choices[0].message.content;
        // dbg!(&res);

        // assert_eq!("", content);
    }
}

// TODO: This requires more updates to the openai Rust library to get the Vision Portion working
pub async fn ask_gpt_vision(
    user_input: String,
    image_url: String,
) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    set_key(env::var("OPENAI_API_KEY").unwrap());

    let text_content = VisionMessage::Text {
        content_type: "text".to_string(),
        text: user_input,
    };

    let image_content = VisionMessage::Image {
        content_type: "image_url".to_string(),
        image_url: ImageUrl { url: image_url },
    };
    let new_content =
        ChatCompletionContent::VisionMessage(vec![text_content, image_content]);

    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(new_content),
        name: None,
        function_call: None,
    }];

    println!("JSON:\n\n {}", serde_json::to_string(&messages).unwrap());
    let model = "gpt-4-vision-preview";

    let chat_completion =
        match ChatCompletion::builder(model.clone(), messages.clone())
            .create()
            .await
        {
            Ok(completion) => completion,
            Err(e) => {
                println!("\n\tChat GPT error occurred: {}", e);
                return Err(e);
            }
        };

    let returned_message =
        chat_completion.choices.first().unwrap().message.clone();

    // TODO: fix
    // println!(
    //     "Chat GPT Response {:#?}: {}",
    //     &returned_message.role,
    //     &returned_message.content.clone().unwrap().trim()
    // );
    Ok(returned_message)
}
