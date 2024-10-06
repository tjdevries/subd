use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::Html,
    routing::get,
    Router,
};
use minijinja::{context, path_loader, Environment};
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use subd_db::get_db_pool;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("./templates"));
    env
});

#[derive(Clone)]
struct AppState {
    pool: Arc<PgPool>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = create_app().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_app() -> Router {
    let pool = get_db_pool().await;

    let state = AppState {
        pool: Arc::new(pool),
    };

    Router::new()
        .route("/", get(root))
        .route("/ai_songs/:id", get(show_ai_song))
        .nest_service("/images", ServeDir::new("./tmp/music_videos"))
        .nest_service("/static", ServeDir::new("./static"))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        )
}

async fn root(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = &state.pool;
    let stats = fetch_stats(pool).await?;
    let unplayed_songs = ai_playlist::get_unplayed_songs(pool).await.unwrap();
    let current_song = ai_playlist::get_current_song(pool).await.ok();
    let current_song_votes_count = if let Some(song) = &current_song {
        ai_songs_vote::total_votes_by_id(pool, song.song_id)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let context = context! {
        stats,
        unplayed_songs,
        current_song,
        current_song_votes_count,
    };

    let tmpl = ENV.get_template("base.html").unwrap();

    let body = tmpl
        .render(context)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(body))
}

async fn show_ai_song(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = &state.pool;
    let stats = fetch_stats(pool).await?;
    let current_song = ai_playlist::find_song_by_id(pool, &id.to_string())
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Song not found".to_string()))?;
    let current_song_votes_count =
        ai_songs_vote::total_votes_by_id(pool, current_song.song_id)
            .await
            .unwrap_or(0);

    let music_directory =
        format!("./tmp/music_videos/{}/", current_song.song_id);
    let images = get_files_by_ext(&music_directory, &["png", "jpg", "jpeg"]);
    let videos = get_files_by_ext(&music_directory, &["mp4"]);

    let image_scores = ai_playlist::models::get_all_image_votes_for_song(
        pool,
        current_song.song_id,
    )
    .await
    .unwrap_or(vec![]);

    let base_path = format!("/images/{}", current_song.song_id);

    let context = context! {
        stats,
        current_song,
        current_song_votes_count,
        images,
        videos,
        image_scores,
        base_path,
    };

    let tmpl = ENV.get_template("song.html").unwrap();

    let body = tmpl
        .render(context)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(body))
}

async fn fetch_stats(pool: &PgPool) -> Result<Stats, (StatusCode, String)> {
    let ai_songs_count = ai_playlist::total_ai_songs(pool).await.unwrap();
    let ai_votes_count = ai_songs_vote::total_votes(pool).await.unwrap();
    let unplayed_songs_count =
        ai_playlist::count_unplayed_songs(pool).await.unwrap();
    Ok(Stats {
        ai_songs_count,
        ai_votes_count,
        unplayed_songs_count,
    })
}

#[derive(Serialize)]
struct Stats {
    ai_songs_count: i64,
    ai_votes_count: i64,
    unplayed_songs_count: i64,
}

fn get_files_by_ext(directory: &str, extensions: &[&str]) -> Vec<String> {
    use std::fs;
    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if extensions.contains(&ext.as_str()) {
                            return path.file_name().map(|name| {
                                name.to_string_lossy().into_owned()
                            });
                        }
                    }
                }
                None
            })
            .collect(),
        Err(_) => vec![],
    }
}
