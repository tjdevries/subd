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
struct MusicVideoScenes {
    scenes: Vec<MusicVideoScene>,
}

#[derive(InstructMacro, Debug, Serialize, Deserialize)]
struct MusicVideoScene {
    image_prompt: String,
    camera_move: String,
}

fn main() {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let prompt = "Describe 5 scenes for a Music Video about based on the following lyrics and their corresponding camera moves:";

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(prompt)),
            name: None,
        }],
    );

    let result = instructor_client
        .chat_completion::<MusicVideoScenes>(req, 3)
        .unwrap();

    println!("{:?}", result);
    // println!("{}", result.name); // John Doe
    // println!("{}", result.age); // 30
}
