use anyhow::Result;
use axum::{
    http::{Method, StatusCode},
    response::Html,
    routing::{get, Router},
};
use std::fs;
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
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Result<Html<String>, (StatusCode, String)> {
    let pool = get_db_pool().await;
    let songs = ai_songs_vote::get_top_songs(&pool, 5)
        .await
        .map_err(|_| "Error getting top songs")
        .unwrap();
    let entries = fs::read_dir("./tmp/fal_images").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading directory: {}", e),
        )
    })?;

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

    let count = ai_playlist::total_ai_songs(&pool).await.unwrap();

    let mut html = String::from(
        "<html>
            <head>
                <style>
                    body {
                        font-family: \"Papyrus\";
                    }
                    .sub-header{
                        font-size: 200%;
                    }

                    .header{
                        font-size: 400%;
                    }
                    .grid-container {
                        display: grid;
                        grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
                        grid-gap: 10px;
                    }
                    .grid-item {
                        text-align: center;
                    }
                </style>
            </head>
            <body>
        ",
    );

    let base_path = "/images";

    html.push_str(&"<h1 class=\"header grid-item\"> AI Top of the Pops</h1>");
    html.push_str(&format!(
        "<h2 class=\"sub-header grid-item\"> Total AI Songs Created: {}</h2>",
        count
    ));
    html.push_str(&format!(
        "<h2 class=\"sub-header grid-item\"> Total AI Song Votes: {}</h2>",
        count
    ));

    html.push_str(&"<h1 class=\"grid-item\"><code class=\"\">!vote 0.0 - 10.0</code></h1>");
    html.push_str(&"<hr />");
    html.push_str("<div class=\"grid-container\">");

    // Soon this will be be top of the pops
    // We would like to display the top 5 songs
    //for song in songs {
    //    html.push_str(&format!("Song: {}", song.title))
    //}

    // Add each image and its ID to the grid
    for (index, image) in images.into_iter().enumerate() {
        html.push_str(&format!(
            "<div class=\"grid-item\">
                <img src=\"{}/{}\" alt=\"{}\" style=\"max-width:400px; max-height:400px;\" /><br/>
                <h1><code>!like {} | !veto {}</code></h1>
            </div>",
            base_path, image, image, index, index
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
