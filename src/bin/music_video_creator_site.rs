use axum::{
    http::{Method, StatusCode},
    response::Html,
    routing::{get, Router},
};
use std::{fs, net::SocketAddr};
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

    // Build our application with routes
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

    // Run our app with hyper, listening globally on port 4001
    let addr = SocketAddr::from(([0, 0, 0, 0], 4001));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler that responds with an HTML page displaying images
async fn root() -> Result<Html<String>, (StatusCode, String)> {
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

    let mut html = String::from("<html><body>\n");

    let base_path = "/images";

    for (index, image) in images.into_iter().enumerate() {
        html.push_str(&format!(
            "<b>{}</b> <img src=\"{}/{}\" alt=\"{}\" style=\"max-width:200px; max-height:200px;\" />\n",
            index, base_path, image, image
        ));
    }
    html.push_str("</body></html>\n");

    Ok(Html(html))
}
