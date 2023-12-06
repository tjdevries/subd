use anyhow::Error;
use anyhow::Result;

use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    completions::Completion,
    set_key,
};
use std::env;

// I want this to exist somewhere else
pub async fn ask_chat_gpt(
    user_input: String,
    base_content: String,
) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    // I use this key
    set_key(env::var("OPENAI_API_KEY").unwrap());

    // but this lib wanted this key
    set_key(env::var("OPENAI_KEY").unwrap());

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

    println!(
        "Chat GPT Response {:#?}: {}",
        &returned_message.role,
        &returned_message.content.clone().unwrap().trim()
    );
    Ok(returned_message)
}

// I want this to exist somewhere else
pub async fn ask_davinci(
    user_input: String,
    base_content: String,
) -> Result<String, anyhow::Error> {
    // ) -> Result<ChatCompletionMessage, openai::OpenAiError> {
    // I use this key
    set_key(env::var("OPENAI_API_KEY").unwrap());

    // but this lib wanted this key
    set_key(env::var("OPENAI_KEY").unwrap());

    // let mut messages = vec![ChatCompletionMessage {
    //     role: ChatCompletionMessageRole::System,
    //     content: Some(base_content),
    //     name: None,
    //     function_call: None,
    // }];
    //
    // messages.push(ChatCompletionMessage {
    //     role: ChatCompletionMessageRole::User,
    //     content: Some(user_input),
    //     name: None,
    //     function_call: None,
    // });

    let prompt = format!("{} {}", base_content, user_input);

    // whats the diff in completion VS  chat completion
    // this is where we pause????
    println!("pre ask_chat_gpt completion");
    // let model = "gpt-4";
    // let model = "gpt-3.5-turbo";
    let model = "text-davinci-003";

    let chat_completion = Completion::builder(model.clone())
        .prompt(prompt)
        .create()
        .await;

    println!("post ask_chat_gpt completion");
    // return chat_completion;
    match chat_completion {
        Ok(chat) => {
            let response = &chat.choices.first().unwrap().text;
            return Ok(response.to_string());
        }
        Err(e) => Err(e.into()),
    }

    // let response = &completion.choices.first().unwrap().text;
    // let chat_completion = match ChatCompletion::builder(model.clone(), messages.clone())
    //     .create()
    //     .await
    // {
    //     Ok(completion) => completion,
    //     Err(e) => {
    //         println!("\n\tChat GPT error occurred: {}", e);
    //         return Err(e);
    //     }
    // };

    // let returned_message =
    //     chat_completion.choices.first().unwrap().message.clone();

    // println!(
    //     "Chat GPT Response {:#?}: {}",
    //     &returned_message.role,
    //     &returned_message.content.clone().unwrap().trim()
    // );
    // Ok(returned_message)
}
