use anyhow::Result;
use chrono::{DateTime, Utc};
use instruct_macros::InstructMacro;
use instruct_macros_types::Parameter;
use instruct_macros_types::{ParameterInfo, StructInfo};
use instructor_ai::from_openai;

use chrono::prelude::*;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};

// We need a different common here
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{self, ChatCompletionRequest},
    common::GPT3_5_TURBO,
    common::GPT4_1106_PREVIEW,
    common::GPT4_O,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub mod dalle;

use openai::{
    chat::{
        ChatCompletion, ChatCompletionContent, ChatCompletionMessage,
        ChatCompletionMessageRole, ImageUrl, VisionMessage,
    },
    set_key,
};

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
struct AIStylesResponse {
    css: String,
}

fn html_file_contents() -> Result<String> {
    let filepath = "../../static/home.html";
    let mut file = File::open(filepath).expect("Failed to open HTML file");
    let mut home_contents = String::new();
    file.read_to_string(&mut home_contents)
        .expect("Failed to read HTML file");

    let filepath = "../../static/songs.html";
    let mut file = File::open(filepath).expect("Failed to open HTML file");
    let mut song_contents = String::new();
    file.read_to_string(&mut song_contents)
        .expect("Failed to read HTML file");

    let filepath = "../../static/users.html";
    let mut file = File::open(filepath).expect("Failed to open HTML file");
    let mut users_contents = String::new();
    file.read_to_string(&mut users_contents)
        .expect("Failed to read HTML file");

    let filepath = "../../static/charts.html";
    let mut file = File::open(filepath).expect("Failed to open HTML file");
    let mut charts_contents = String::new();
    file.read_to_string(&mut charts_contents)
        .expect("Failed to read HTML file");

    let contents = format!(
        "{} {} {} {}",
        home_contents, song_contents, users_contents, charts_contents
    );
    Ok(contents)
}

fn get_music_video_scene() {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap());
    let instructor_client = from_openai(client);

    let contents = html_file_contents().unwrap();

    let css_base_prompt =
        "Generate excellent high-quality detailed CSS for the provided HTML. Provide as many CSS properties as possible.";
    let css_style_tips = "Make the overall style consistent, with a color theme and font-selection that cohesive but fun.";
    let css_properties_prompt = "Use as many CSS properties as possible, for example: animation-duration, animation-delay, animation-direction, animation-fill-mode, animation, clip-path, mix-blend-mode, backdrop-filter, filter, text-shadow, box-shadow, border-image, mask, background-clip, scroll-snap-type, transform, perspective, isolation, object-fit, object-position, animation, transition, shape-outside, will-change, overflow-anchor, conic-gradient, linear-gradient, radial-gradient, font-variant, text-stroke, aspect-ratio, grid-template-areas, scroll-behavior, pointer-events, scroll-margin, scroll-padding, gap, align-self, justify-items, object-position, word-wrap, hyphens, resize, appearance, backface-visibility, scrollbar-color, scrollbar-width, font-feature-settings, text-orientation, column-count, column-gap, column-rule, blend-mode, background-origin, font-display, grid-auto-flow, grid-template-rows etc.";
    let css_interactions =
        "Include as many interactive changes as possible, like on:hover. Make all links do something bold and dynamic on hover.";
    let css_layouts = "Also feel free to try different layouts using things like display: grid.";
    let css_fonts = "Use a wide variety of bold unique fonts, which you need to specify to import.";
    let css_animations = "Use as many animations and transitions as possible.";
    let css_final_prompt =
        "Make it all savaveable as a styles.css file, for the following HTML: ";
    let spinner_css =
        "Make sure you use things like transform: rotate(360deg); Prioritize movement with transform.";

    let prompt = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        css_base_prompt,
        css_properties_prompt,
        css_interactions,
        css_layouts,
        css_fonts,
        css_final_prompt,
        css_style_tips,
        css_animations,
        spinner_css,
        contents
    );

    let req = ChatCompletionRequest::new(
        // GPT3_5_TURBO.to_string(),
        GPT4_O.to_string(),
        // GPT4_1106_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(prompt),
            name: None,
        }],
    );

    let result = instructor_client
        .chat_completion::<AIStylesResponse>(req, 3)
        .expect("Failed to get chat completion");

    println!("{:?}", result);

    // Backup the existing styles.css file
    let now = Utc::now();
    let timestamp = now.format("%Y%m%d%H%M%S").to_string();
    let backup_filename = format!("../../static/{}.css", timestamp);

    let src = "../../static/styles.css";
    let dest = &backup_filename;

    fs::copy(src, dest).expect("Failed to backup the styles.css file");

    // Save the new CSS to styles.css
    let content = &result.css;

    let mut file = File::create("../../static/styles.css")
        .expect("Failed to create styles.css file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to styles.css file");
}

// I want this to exist somewhere else
// probably in a Crate
pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage, String> {
    set_key(env::var("OPENAI_API_KEY").unwrap());

    let base_message = ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(ChatCompletionContent::Message(Some(base_content))),
        name: None,
        function_call: None,
    };

    let mut messages = vec![base_message];

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(ChatCompletionContent::Message(Some(user_input))),
        name: None,
        function_call: None,
    });

    // let model = "gpt-4";
    let model = "gpt-3.5-turbo";

    let chat_completion = match ChatCompletion::builder(model, messages.clone())
        .create()
        .await
    {
        Ok(completion) => completion,
        Err(e) => {
            println!("\n\tChat GPT error occurred: {}", e);
            return Err(e.to_string());
        }
    };

    chat_completion
        .choices
        .first()
        .ok_or("Error Finding first Chat GPT response".to_string())
        .map(|m| m.message.clone())
}

pub async fn ask_gpt_vision2(
    image_path: &str,
    image_url: Option<&str>,
) -> Result<String, anyhow::Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");

    let full_path = match image_url {
        Some(url) => url.to_string(),
        None => {
            let base64_image = subd_image_utils::encode_image(image_path)?;
            format!("data:image/jpeg;base64,{}", base64_image)
        }
    };

    let client = reqwest::Client::new();

    let content_type = "application/json".parse()?;
    let auth = format!("Bearer {}", api_key).parse()?;
    let h = vec![
        (reqwest::header::CONTENT_TYPE, content_type),
        (reqwest::header::AUTHORIZATION, auth),
    ];

    let headers = reqwest::header::HeaderMap::from_iter(h);

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

    let now: DateTime<Utc> = Utc::now();
    let filename = format!("{}.json", now.timestamp());
    let filepath =
        format!("/home/begin/code/subd/tmp/Archive/vision/{}", filename);
    let mut file = File::create(filepath)?;
    file.write_all(response_json.to_string().as_bytes())?;

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

// TODO: This requires more updates to the openai Rust library to get the Vision Portion working
pub async fn ask_gpt_vision(
    user_input: String,
    image_url: String,
) -> Result<ChatCompletionMessage, String> {
    let key = env::var("OPENAI_API_KEY").map_err(|e| e.to_string())?;
    set_key(key);

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

    let debug = serde_json::to_string(&messages)
        .unwrap_or("Couldn't parse gpt vision message".to_string());
    println!("GPT Vision:\n\n {}", debug);
    let model = "gpt-4-vision-preview";

    let chat_completion = match ChatCompletion::builder(model, messages.clone())
        .create()
        .await
    {
        Ok(completion) => completion,
        Err(e) => {
            println!("\n\tChat GPT error occurred: {}", e);
            return Err(e.to_string());
        }
    };

    chat_completion
        .choices
        .first()
        .ok_or("Error finding GPT Vision first response".to_string())
        .map(|m| m.message.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[tokio::test]
    async fn test_telephone() {
        // let first_image = "https://d23.com/app/uploads/2013/04/1180w-600h_mickey-mouse_1.jpg";
        // let first_image = "https://mario.wiki.gallery/images/thumb/1/13/Funky_Kong_Artwork_-_Donkey_Kong_Country_Tropical_Freeze.png/600px-Funky_Kong_Artwork_-_Donkey_Kong_Country_Tropical_Freeze.png";
        // let first_image = "https://static.wikia.nocookie.net/donkeykong/images/7/72/Candy.PNG/revision/latest/scale-to-width-down/110?cb=20130203073312";
        // let first_image = "https://www.tbstat.com/wp/uploads/2023/05/Fvz9hOIXwAEaIR8-669x675.jpeg";
        let _first_image = "https://upload.wikimedia.org/wikipedia/en/thumb/3/3b/SpongeBob_SquarePants_character.svg/1200px-SpongeBob_SquarePants_character.svg.png";

        // let res = telephone(first_image.to_string(), "more chill".to_string(), 10).await.unwrapa);
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
    #[ignore]
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

    // We might want to try more structured version
    #[tokio::test]
    async fn test_generating_css() {
        //let filepath = "../../tmp/music_video_creator.html";
        //let mut file = File::open(filepath).unwrap();
        //let mut contents = String::new();
        //let _ = file.read_to_string(&mut contents);
        //// Now we need to generate CSS for styles
        //assert!(true);
        //let res = ask_chat_gpt(
        //    "Generate FUN Interesting CSS that is savable as a styles.css file, for the following HTML: ".to_string(),
        //    contents,
        //)
        //.await;
        //println!("{:?}", res);
        get_music_video_scene();
    }
}
