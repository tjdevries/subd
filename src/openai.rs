use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};
use std::env;

// I want this to exist somewhere else
pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    set_key(env::var("OPENAI_API_KEY").unwrap());

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(base_content),
        name: None,
        function_call: None,
    }];

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(user_input),
        name: None,
        function_call: None,
    });

    // this is where we pause????
    println!("pre ask_chat_gpt completion");
    // let model = "gpt-4";
    let model = "gpt-3.5-turbo";
    let chat_completion = match ChatCompletion::builder(model, messages.clone())
        .create()
        .await
    {
        Ok(completion) => completion,
        Err(e) => {
            eprintln!("\n\tChat GPT error occurred: {}", e);
            return Err(e);
        }
    };
    println!("post ask_chat_gpt completion");

    let returned_message =
        chat_completion.choices.first().unwrap().message.clone();

    println!(
        "Chat GPT Response {:#?}: {}",
        &returned_message.role,
        &returned_message.content.clone().unwrap().trim()
    );
    Ok(returned_message)
}
