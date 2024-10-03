use anyhow::Result;
use axum::{
    http::{Method, StatusCode},
    response::Html,
    routing::{get, Router},
};
use std::{fs, net::SocketAddr};
use subd_db::get_db_pool;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Build our static file service
    let serve_dir = ServeDir::new("./tmp/fal_images");

    let app = Router::new()
        .route("/", get(root))
        // Serve static files from "./tmp/fal_images" at the "/images" path
        .nest_service("/images", serve_dir)
        // Add CORS layer if needed
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // Allow any origin
                .allow_methods([Method::GET]),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler that responds with an HTML page displaying images in a grid
async fn root() -> Result<Html<String>, (StatusCode, String)> {
    let pool = get_db_pool().await;
    let songs = ai_songs_vote::get_top_songs(&pool, 5)
        .await
        .map_err(|_| "Error getting top songs")
        .unwrap();
    // Read the "./tmp/fal_images" directory
    let entries = fs::read_dir("./tmp/fal_images").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading directory: {}", e),
        )
    })?;

    // Collect image file names
    let images = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        return path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>();

    // Start building the HTML content
    let mut html = String::from(
        "<html>
            <head>
                <style>
                    .grid-container {
                        display: grid;
                        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                        grid-gap: 10px;
                    }
                    .grid-item {
                        text-align: center;
                    }
                </style>
            </head>
            <body>
                <div class=\"grid-container\">
        ",
    );

    let base_path = "/images";

    // We need to display the top 5 songs
    for song in songs {
        html.push_str(&format!("Song: {}", song.title))
    }

    // Add each image and its ID to the grid
    for (index, image) in images.into_iter().enumerate() {
        html.push_str(&format!(
            "<div class=\"grid-item\">
                <img src=\"{}/{}\" alt=\"{}\" style=\"max-width:200px; max-height:200px;\" /><br/>
                <b>{}</b>
            </div>",
            base_path, image, image, index
        ));
    }

    // Close the HTML tags
    html.push_str(
        "       </div>
            </body>
        </html>",
    );

    Ok(Html(html))
}
