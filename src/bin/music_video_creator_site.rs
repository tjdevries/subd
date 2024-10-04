use anyhow::Result;
use axum::{
    extract::Extension, extract::FromRef, extract::State, http::StatusCode,
    response::IntoResponse, routing::get, routing::post, Json, Router,
};
use axum::{http::Method, response::Html};
use std::fs;
use std::{net::SocketAddr, sync::Arc};
use subd_db::get_db_pool;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber;

#[derive(Clone, FromRef)]
struct AppState {
    pool: Arc<sqlx::PgPool>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let serve_dir = ServeDir::new("./tmp/music_videos");
    let pool = get_db_pool().await;
    let state = AppState {
        pool: Arc::new(pool),
    };

    let app = Router::new()
        .route("/", get(root))
        // Serve static files from "./tmp/fal_images" at the "/images" path
        .nest_service("/images", serve_dir)
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(
    State(pool): State<Arc<sqlx::PgPool>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut html = String::from(
        "<html>
            <head>
            
                <meta http-equiv=\"refresh\" content=\"1\" />
                <style>
                    body {
                        font-family: \"Papyrus\";
                    }
                    .sub-header{
                        font-size: 200%;
                    }

                    .current-song {
                        padding: 10px;
                        border: 2px solid black;
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
    let count = ai_playlist::total_ai_songs(&pool).await.unwrap();
    // let songs = ai_songs_vote::get_top_songs(&pool, 5)
    //     .await
    //     .map_err(|_| "Error getting top songs")
    //     .unwrap();

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

    let current_song = ai_playlist::get_current_song(&pool)
        .await
        .map_err(|_| "Error getting current song");

    if let Ok(current_song) = current_song {
        // We need to just leave this loop
        let music_directory =
            format!("./tmp/music_videos/{}/", current_song.song_id);
        if !fs::metadata(&music_directory).is_ok() {
            fs::create_dir_all(&music_directory).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error creating directory: {}", e),
                )
            })?;
        }

        let entries = fs::read_dir(&music_directory).map_err(|e| {
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
                            return path.file_name().map(|name| {
                                name.to_string_lossy().into_owned()
                            });
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        let base_path = format!("/images/{}", current_song.song_id);
        html.push_str(&format!(
            "<h2 class=\"sub-header grid-item current-song\"> Current Song: {} | Tags: {} | Creator: @{} | {}</h2>",
            current_song.title, current_song.tags, current_song.username, current_song.song_id
        ));

        html.push_str("<div class=\"grid-container\">");

        // only need this if we have a current_song
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
    }

    // Soon this will be be top of the pops
    // We would like to display the top 5 songs
    // html.push_str(&"<hr />");
    // for song in songs {
    //     html.push_str(&format!("Song: {}", song.title))
    // }
    // html.push_str(&"<hr />");

    // Close the HTML tags
    html.push_str(
        "       </div>
            </body>
        </html>",
    );

    Ok(Html(html))
}
