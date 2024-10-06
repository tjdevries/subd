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
struct UserInfo {
    // This represents the name of the user
    image_prompts: Vec<String>,
    // This represents the age of the user
    // age: u8,
}

fn main() {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let instructor_client = from_openai(client);

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from(
                "Describe 5 image_prompts for a Music Video about Programming in Rust",
            )),
            name: None,
        }],
    );

    let result = instructor_client
        .chat_completion::<UserInfo>(req, 3)
        .unwrap();

    println!("{:?}", result);
    // println!("{}", result.name); // John Doe
    // println!("{}", result.age); // 30
}
