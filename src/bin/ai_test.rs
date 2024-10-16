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
    let _ = save_embedding_from_file().await;
    let _ = ask_question().await;
}

async fn ask_question() -> Result<()> {
    let result_js = generate_js().await;
    if let Err(e) = result_js {
        println!("Error generating JS: {}", e);
    }

    let result_css = generate_css().await;
    if let Err(e) = result_css {
        println!("Error generating CSS: {}", e);
    }
    Ok(())
}

async fn generate_js() -> Result<()> {
    let embedding_model = TextEmbeddingAda002;
    let store =
        PostgresVectorStore::try_new("embeddings", embedding_model).await?;

    let embeddings = vec![];
    store.store_batch(embeddings).await?;

    let html_to_animate_folder = "./templates";
    let contents = subd_openai::ai_styles::html_file_contents(Some(
        html_to_animate_folder,
    ))?;

    let embedding_client = OpenAIEmbeddingClient::try_new(TextEmbeddingAda002)?;
    let retriever =
        store.as_retriever(embedding_client, DistanceFunction::Cosine);
    // let chat_client = OpenAIChatCompletionClient::try_new(Gpt3Point5Turbo)?;
    let chat_client = OpenAIChatCompletionClient::try_new(Gpt4o)?;

    let js_expert_prompt = fs::read_to_string("./prompts/js_expert.txt")?;
    let system_prompt = PromptMessage::SystemMessage(js_expert_prompt);

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
        "Generate JS for the following HTML: {}",
        contents
    ));

    let response = chain
        .invoke_chain(user_message, NonZeroU32::new(2).unwrap())
        .await?;

    println!("{}", response.content());
    let filtered_content = response
        .content()
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<&str>>()
        .join("\n");

    fs::write("./static/styles.js", filtered_content)?;
    Ok(())
}

async fn generate_css() -> Result<()> {
    let embedding_model = TextEmbeddingAda002;
    let store =
        PostgresVectorStore::try_new("embeddings", embedding_model).await?;

    let embeddings = vec![];
    store.store_batch(embeddings).await?;

    let html_to_animate_folder = "./templates";
    let contents = subd_openai::ai_styles::html_file_contents(Some(
        html_to_animate_folder,
    ))?;

    let embedding_client = OpenAIEmbeddingClient::try_new(TextEmbeddingAda002)?;
    let retriever =
        store.as_retriever(embedding_client, DistanceFunction::Cosine);
    let chat_client = OpenAIChatCompletionClient::try_new(Gpt4o)?;
    //let chat_client = OpenAIChatCompletionClient::try_new(Gpt3Point5Turbo)?;

    let css_expert_prompt = fs::read_to_string("./prompts/css_expert.txt")?;
    let system_prompt = PromptMessage::SystemMessage(css_expert_prompt);

    let chain: BasicRAGChain<
        OpenAIChatCompletionClient,
        PostgresVectorRetriever<_>,
    > = BasicRAGChain::builder()
        .system_prompt(system_prompt)
        .chat_client(chat_client)
        .retriever(retriever)
        .build();

    let user_message = PromptMessage::HumanMessage(format!(
        "Generate CSS for the following HTML: {}",
        contents
    ));

    let response = chain
        .invoke_chain(user_message, NonZeroU32::new(2).unwrap())
        .await?;

    println!("{}", response.content());
    let filtered_content = response
        .content()
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<&str>>()
        .join("\n");

    fs::write("./static/styles.css", filtered_content)?;
    Ok(())
}

async fn save_embedding_from_file() -> Result<()> {
    const EMBEDDING_MODEL: OpenAIEmbeddingModel =
        OpenAIEmbeddingModel::TextEmbeddingAda002;

    let text = std::fs::read_to_string("./tmp/example_text.txt")?;

    let chunker = TokenChunker::try_new(
        std::num::NonZeroUsize::new(50).unwrap(),
        25,
        EMBEDDING_MODEL,
    )?;
    let chunks: Chunks = chunker.generate_chunks(&text)?;

    let store: PostgresVectorStore =
        PostgresVectorStore::try_new("embeddings", EMBEDDING_MODEL).await?;

    let client: OpenAIEmbeddingClient =
        OpenAIEmbeddingClient::try_new(EMBEDDING_MODEL)?;
    let embeddings: Vec<Embedding> = client.generate_embeddings(chunks).await?;

    store.store_batch(embeddings).await?;
    Ok(())
}
