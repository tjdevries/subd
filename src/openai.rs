use std::env;
use openai::{
    chat::{ ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};

// I want this to exist somewhere else
pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    println!("pre ask_chat_gpt OPENAI_KEY");
    set_key(env::var("OPENAI_API_KEY").unwrap());
    println!("post ask_chat_gpt OPENAI_KEY)");

    println!("pre ask_chat_gpt messages");
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(base_content),
        name: None,
        function_call: None,
    }];
    println!("post ask_chat_gpt messages");
    
    println!("pre ask_chat_gpt message push");
    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(user_input),
        name: None,
        function_call: None,
    });
    println!("post ask_chat_gpt message push");
    
    println!("pre ask_chat_gpt completion");
    // let model = "gpt-4";
    let model="gpt-3.5-turbo";
    let chat_completion = match ChatCompletion::builder(model, messages.clone()).create().await {
        Ok(completion) => completion,
        Err(e) => {
            eprintln!("An error occurred: {}", e);
            return Err(e);
        }
    };
    println!("post ask_chat_gpt completion");
    
    println!("pre ask_chat_gpt completion choices");
    let returned_message =
        chat_completion.choices.first().unwrap().message.clone();
    println!("post ask_chat_gpt completion choices");
    
    println!(
        "Chat GPT Response {:#?}: {}",
        &returned_message.role,
        &returned_message.content.clone().unwrap().trim()
    );
    Ok(returned_message)
}

