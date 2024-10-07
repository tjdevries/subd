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

    // TODO: We need to rename the routes here
    // We also need to unify all the good collecting data from the database
    Router::new()
        .route("/", get(home))
        .route("/ai_songs/:id", get(show_ai_song))
        .route("/users/:username", get(show_ai_song_by_user))
        .route("/all_songs", get(show_ai_songs))
        .nest_service("/images", ServeDir::new("./tmp/music_videos"))
        .nest_service("/songs", ServeDir::new("./ai_songs"))
        .nest_service("/static", ServeDir::new("./static"))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        )
}

async fn home(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = &state.pool;
    let stats = ai_songs_vote::fetch_stats(pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let unplayed_songs = ai_playlist::get_unplayed_songs(pool)
        .await
        .unwrap_or(vec![]);

    let current_song_info = ai_songs_vote::get_current_song_info(pool)
        .await
        .unwrap_or(ai_songs_vote::CurrentSongInfo {
            current_song: None,
            votes_count: 0,
        });

    let (videos, image_scores) = ai_playlist::get_videos_and_image_scores(
        pool,
        &current_song_info.current_song,
    )
    .await;

    let users = ai_songs_vote::get_users_with_song_count_and_avg_score(pool)
        .await
        .unwrap();
    println!("Image scores: {:?}", image_scores);

    let score = match current_song_info.current_song {
        Some(song) => {
            match ai_songs_vote::get_average_score(pool, song.song_id).await {
                Ok(result) => result.avg_score.to_string(),
                Err(_) => "No Votes".to_string(),
            }
        }
        None => "No Votes".to_string(),
    };

    // This is sooo stupid
    // we are getting the current_song as an option again, because it's consumed above
    let current_song = ai_playlist::get_current_song(pool).await.ok();
    let votes_count = current_song_info.votes_count;
    let context = context! {
        score,
        videos,
        image_scores,
        users,
        stats,
        unplayed_songs,
        current_song,
        votes_count,
    };

    let tmpl = ENV.get_template("home.html").unwrap();

    let body = tmpl
        .render(context)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(body))
}

async fn show_ai_songs(
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = &state.pool;
    let stats = ai_songs_vote::fetch_stats(pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let songs = ai_playlist::all_songs(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let context = context! {
        songs,
        stats,
    };

    let tmpl = ENV.get_template("songs.html").unwrap();

    let body = tmpl
        .render(context)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(body))
}

async fn show_ai_song_by_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Html<String>, (StatusCode, String)> {
    let pool = &state.pool;
    let songs = ai_playlist::models::get_songs_for_user(pool, &username)
        .await
        .unwrap();
    let stats = ai_songs_vote::fetch_stats(pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let context = context! {stats, songs};
    let tmpl = ENV.get_template("songs.html").unwrap();
    //
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
    let stats = ai_songs_vote::fetch_stats(pool)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let current_song = ai_playlist::find_song_by_id(pool, &id.to_string())
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Song not found".to_string()))?;
    let current_song_votes_count =
        ai_songs_vote::total_votes_by_id(pool, current_song.song_id)
            .await
            .unwrap_or(0);

    let music_directory =
        format!("./tmp/music_videos/{}/", current_song.song_id);
    let images =
        subd_utils::get_files_by_ext(&music_directory, &["png", "jpg", "jpeg"]);
    let videos = subd_utils::get_files_by_ext(&music_directory, &["mp4"]);

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
