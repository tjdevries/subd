use anyhow::Result;
use rag_toolchain::chains::BasicRAGChain;
use rag_toolchain::chunkers::{Chunker, TokenChunker};
use rag_toolchain::clients::AsyncEmbeddingClient;
use rag_toolchain::clients::OpenAIChatCompletionClient;
use rag_toolchain::clients::OpenAIEmbeddingClient;
use rag_toolchain::clients::OpenAIModel::Gpt3Point5Turbo;
use rag_toolchain::clients::PromptMessage;
use rag_toolchain::common::OpenAIEmbeddingModel::TextEmbeddingAda002;
use rag_toolchain::common::{Chunks, Embedding, OpenAIEmbeddingModel};
use rag_toolchain::retrievers::DistanceFunction;
use rag_toolchain::retrievers::PostgresVectorRetriever;
use rag_toolchain::stores::{EmbeddingStore, PostgresVectorStore};
use std::fs;
use std::num::NonZeroU32;

const SYSTEM_MESSAGE: &'static str =
"You are to give straight forward answers using the supporting information you are provided";

#[tokio::main]
async fn main() {
    let _ = save_embedding_from_file().await;
    let _ = ask_question().await;
}

async fn ask_question() {
    // USe the RAG to get into
    // Use the simple Query to get it structured

    //let embedding_model: OpenAIEmbeddingModel =
    //    OpenAIEmbeddingModel::TextEmbedding3Small;
    let embedding_model = TextEmbeddingAda002;
    // Initialize the PostgresVectorStore
    let store: PostgresVectorStore =
        PostgresVectorStore::try_new("embeddings", embedding_model)
            .await
            .unwrap();

    let embeddings = vec![];

    println!("hmmm");
    store.store_batch(embeddings).await.unwrap();

    let html_to_animate_folder = "./templates";
    let contents = subd_openai::ai_styles::html_file_contents(Some(
        html_to_animate_folder,
    ))
    .unwrap();
    // Create a new embedding client
    let embedding_client: OpenAIEmbeddingClient =
        OpenAIEmbeddingClient::try_new(TextEmbeddingAda002).unwrap();

    // Convert our store into a retriever
    let retriever: PostgresVectorRetriever<OpenAIEmbeddingClient> =
        store.as_retriever(embedding_client, DistanceFunction::Cosine);

    // Create a new chat client
    let chat_client: OpenAIChatCompletionClient =
        OpenAIChatCompletionClient::try_new(Gpt3Point5Turbo).unwrap();

    // Define our system prompt
    let css_expert_prompt = fs::read_to_string("./prompts/css_expert.txt")
        .expect("Failed to read file");
    let system_prompt: PromptMessage =
        PromptMessage::SystemMessage(css_expert_prompt.into());
    //let system_prompt: PromptMessage =
    //    PromptMessage::SystemMessage(SYSTEM_MESSAGE.into());

    // Create a new BasicRAGChain with over our open ai chat client and postgres vector retriever
    let chain: BasicRAGChain<
        OpenAIChatCompletionClient,
        PostgresVectorRetriever<_>,
    > = BasicRAGChain::builder()
        .system_prompt(system_prompt)
        .chat_client(chat_client)
        .retriever(retriever)
        .build();
    // Define our user prompt
    let user_message: PromptMessage = PromptMessage::HumanMessage(format!(
        "Generate CSS for the following HTML: {}",
        contents
    ));

    // Invoke the chain. Under the hood this will retrieve some similar text from
    // the retriever and then use the chat client to generate a response.
    let response = chain
        .invoke_chain(user_message, NonZeroU32::new(2).unwrap())
        .await
        .unwrap();

    println!("{}", response.content());
    let filtered_content = response
        .content()
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .collect::<Vec<&str>>()
        .join("\n");
    fs::write("./static/styles.css", filtered_content)
        .expect("Failed to write CSS file");
}

async fn save_embedding_from_file() -> Result<()> {
    const EMBEDDING_MODEL: OpenAIEmbeddingModel =
        OpenAIEmbeddingModel::TextEmbeddingAda002;

    // We read in the text from a file
    let text = std::fs::read_to_string("./tmp/example_text.txt").unwrap();

    // Create a new chunker and generate the chunks
    let chunker = TokenChunker::try_new(
        std::num::NonZeroUsize::new(50).unwrap(),
        25,
        EMBEDDING_MODEL,
    )
    .unwrap();
    let chunks: Chunks = chunker.generate_chunks(&text).unwrap();

    // I would check your store initialized before sending of embeddings to openai
    let store: PostgresVectorStore =
        PostgresVectorStore::try_new("embeddings", EMBEDDING_MODEL)
            .await
            .unwrap();

    // Create a new client and generate the embeddings for the chunks
    let client: OpenAIEmbeddingClient =
        OpenAIEmbeddingClient::try_new(EMBEDDING_MODEL).unwrap();
    let embeddings: Vec<Embedding> =
        client.generate_embeddings(chunks).await.unwrap();

    // Insert the embeddings into the store
    store.store_batch(embeddings).await.unwrap();
    Ok(())
}
