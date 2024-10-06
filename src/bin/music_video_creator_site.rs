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

#[derive(Serialize)]
struct Stats {
    ai_songs_count: i64,
    ai_votes_count: i64,
    unplayed_songs_count: i64,
}

// We need to figure this out
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
        .route("/users/:username", get(show_ai_song_by_user))
        .route("/songs", get(show_ai_songs))
        .nest_service("/images", ServeDir::new("./tmp/music_videos"))
        .nest_service("/all_songs", ServeDir::new("./ai_songs"))
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

    // Only if this is something
    let (videos, image_scores) = if let Some(song) = &current_song {
        let music_directory = format!("./tmp/music_videos/{}/", song.song_id);

        let ids = subd_utils::get_files_by_ext(
            &music_directory,
            &["png", "jpg", "jpeg"],
        )
        .iter()
        .map(|path| path.to_string())
        .collect::<Vec<String>>();
        let videos = subd_utils::get_files_by_ext(&music_directory, &["mp4"]);

        // We actually need the extension
        // let image_scores = ai_playlist::models::get_all_image_votes_for_song(
        let image_scores =
            // ai_playlist::models::get_image_votes_or_default(pool, ids)
            ai_playlist::models::get_image_votes_or_default_with_extensions(pool, ids)
                .await
                .unwrap_or_default();
        // let image_scores = vec![];

        (videos, image_scores)
    } else {
        (vec![], vec![("".to_string(), "".to_string(), 0, 0)])
    };

    // let base_path = format!("/images/{}", current_song.song_id);
    let users = ai_playlist::get_users_with_song_count(&pool).await.unwrap();
    println!("Image scores: {:?}", image_scores);
    // println!("Images: {:?}", images);

    // We can't use the current song here
    let context = context! {
        videos,
        image_scores,
        users,
        stats,
        unplayed_songs,
        current_song,
        current_song_votes_count,
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
    let stats = fetch_stats(pool).await?;

    // This must be failing
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
    let songs = ai_playlist::models::get_songs_for_user(&pool, &username)
        .await
        .unwrap();
    let stats = fetch_stats(pool).await?;
    //let current_song = ai_playlist::find_song_by_id(pool, &id.to_string())
    //    .await
    //    .map_err(|_| (StatusCode::NOT_FOUND, "Song not found".to_string()))?;
    //let current_song_votes_count =
    //    ai_songs_vote::total_votes_by_id(pool, current_song.song_id)
    //        .await
    //        .unwrap_or(0);
    //
    //let music_directory =
    //    format!("./tmp/music_videos/{}/", current_song.song_id);
    //let images =
    //    subd_utils::get_files_by_ext(&music_directory, &["png", "jpg", "jpeg"]);
    //let videos = subd_utils::get_files_by_ext(&music_directory, &["mp4"]);
    //
    //let image_scores = ai_playlist::models::get_all_image_votes_for_song(
    //    pool,
    //    current_song.song_id,
    //)
    //.await
    //.unwrap_or(vec![]);
    //
    //let base_path = format!("/images/{}", current_song.song_id);
    //
    //let context = context! {
    //    stats,
    //    current_song,
    //    current_song_votes_count,
    //    images,
    //    videos,
    //    image_scores,
    //    base_path,
    //};
    //
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
