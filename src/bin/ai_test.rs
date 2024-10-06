// use kalosm::language::*;

// First, derive an efficient parser for your structured data
// #[derive(Parse, Clone, Debug)]
// enum Class {
//     Thing,
//     Person,
//     Animal,
// }
//
// #[derive(Parse, Clone, Debug)]
// struct Response {
//     classification: Class,
// }

#[tokio::main]
async fn main() {
    // let base_url = std::env::var("OPENAI_API_BASE")
    //     .expect("Custom OPENAI_API_BASE not set");
    // let llm = Gpt3_5::builder().with_base_url(&base_url).build();
    //
    // let task = Task::builder("You classify the user's message as about a person, animal or thing in a JSON format")
    //     .with_constraints(Response::new_parser())
    //     .build();
    // let response = task.run("The Kalosm library lets you create structured data from natural language inputs", &llm).await.unwrap();

    println!("  The Kalosm library lets you create structured data from natural language inputs");
    // println!("{:?}", response);

    // let prompt = "The following is a 300 word essay about why the capital of France is Paris:";
    // print!("{}", prompt);
    //
    // let mut stream =
    //     llm.stream_text(prompt).with_max_length(300).await.unwrap();
    // stream.to_std_out().await.unwrap();

    // // Then set up a task with a prompt and constraints
    // let llm = Llama::new_chat().await.unwrap();
    // let task = Task::builder("You classify the user's message as about a person, animal or thing in a JSON format")
    //     .with_constraints(Response::new_parser())
    //     .build();
    //
    // // Finally, run the task
    // let response = task.run("The Kalosm library lets you create structured data from natural language inputs", &llm).await.unwrap();
}
