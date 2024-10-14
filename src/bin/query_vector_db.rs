use anyhow::Result;
use rag_toolchain::chains::BasicRAGChain;
use rag_toolchain::chunkers::{Chunker, TokenChunker};
use rag_toolchain::clients::AsyncEmbeddingClient;
use rag_toolchain::clients::OpenAIChatCompletionClient;
use rag_toolchain::clients::OpenAIEmbeddingClient;
//use rag_toolchain::clients::OpenAIModel::Gpt3Point5Turbo;
use rag_toolchain::clients::OpenAIModel::Gpt4o;
use rag_toolchain::clients::PromptMessage;
use rag_toolchain::common::OpenAIEmbeddingModel::TextEmbeddingAda002;
use rag_toolchain::common::{Chunks, Embedding, OpenAIEmbeddingModel};
use rag_toolchain::retrievers::DistanceFunction;
use rag_toolchain::retrievers::PostgresVectorRetriever;
use rag_toolchain::stores::{EmbeddingStore, PostgresVectorStore};
use std::fs;
use std::num::NonZeroU32;

#[tokio::main]
async fn main() {
    println!("It's time");
    // let username = "zanuss".to_string();
    let username = "carlvandergeest".to_string();
    let _ = ask_question(username).await;
}

async fn ask_question(username: String) -> Result<()> {
    let result_js = query_for_username(username).await;
    if let Err(e) = result_js {
        println!("Error generating JS: {}", e);
    }
    Ok(())
}

async fn query_for_username(username: String) -> Result<()> {
    let embedding_model = TextEmbeddingAda002;
    let store =
        PostgresVectorStore::try_new("embeddings", embedding_model).await?;

    let embedding_client = OpenAIEmbeddingClient::try_new(TextEmbeddingAda002)?;
    let retriever =
        store.as_retriever(embedding_client, DistanceFunction::Cosine);
    // let chat_client = OpenAIChatCompletionClient::try_new(Gpt3Point5Turbo)?;
    let chat_client = OpenAIChatCompletionClient::try_new(Gpt4o)?;

    let base_prompt = fs::read_to_string("./prompts/twitch_user.txt")?;
    let prompt = format!("{} {}", base_prompt, username);
    let system_prompt = PromptMessage::SystemMessage(prompt.into());

    let chain: BasicRAGChain<
        OpenAIChatCompletionClient,
        PostgresVectorRetriever<_>,
    > = BasicRAGChain::builder()
        .system_prompt(system_prompt)
        .chat_client(chat_client)
        .retriever(retriever)
        .build();

    println!("Prompting");
    let user_message = PromptMessage::HumanMessage(format!(
        "What are you're primary interests, be specific",
    ));

    let response = chain
        .invoke_chain(user_message, NonZeroU32::new(2).unwrap())
        .await?;

    println!("{}", response.content());
    Ok(())
}
