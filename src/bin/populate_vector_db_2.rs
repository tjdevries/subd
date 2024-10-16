use anyhow::Result;
use rag_toolchain::chunkers::{Chunker, TokenChunker};
use rag_toolchain::clients::AsyncEmbeddingClient;
use rag_toolchain::clients::OpenAIEmbeddingClient;
use rag_toolchain::common::{Chunks, OpenAIEmbeddingModel};
use rag_toolchain::stores::{EmbeddingStore, PostgresVectorStore};

#[tokio::main]
async fn main() {
    println!("It's populate our embeddings DB");
    // let username = "zanuss";
    let username = "carlvandergeest";
    // let username = "carlvandergeest";
    let res = save_embedding_from_file(username).await;
    if let Err(e) = res {
        println!("Error: {}", e);
    }
}

async fn save_embedding_from_file(username: &str) -> Result<()> {
    let pool = subd_db::get_db_pool().await;
    let database_name = "embeddings";

    let messages =
        user_service::models::user_messages::Model::get_messages_by_username(
            username, &pool,
        )
        .await
        .unwrap();

    println!("Messages Count: {:?}", messages.len());

    const EMBEDDING_MODEL: OpenAIEmbeddingModel =
        OpenAIEmbeddingModel::TextEmbeddingAda002;

    let store: PostgresVectorStore =
        PostgresVectorStore::try_new(database_name, EMBEDDING_MODEL).await?;

    for message in messages {
        let vector_msg = format!("{}: {}", username, message.contents);

        let chunker = TokenChunker::try_new(
            std::num::NonZeroUsize::new(50).unwrap(),
            25,
            EMBEDDING_MODEL,
        )?;

        println!("Chunking");
        let chunks: Chunks = chunker.generate_chunks(&vector_msg)?;
        println!("Done Chunking");

        let client: OpenAIEmbeddingClient =
            OpenAIEmbeddingClient::try_new(EMBEDDING_MODEL)?;
        println!("Generating new Embeddings");
        let new_embeddings = match client.generate_embeddings(chunks).await {
            Ok(embeddings) => embeddings,
            Err(e) => {
                println!("Error generating embeddings: {:?}", e);
                return Err(e.into());
            }
        };
        println!("Embeddings Count: {:?}", new_embeddings.len());
        store.store_batch(new_embeddings).await?;
    }
    Ok(())
}
