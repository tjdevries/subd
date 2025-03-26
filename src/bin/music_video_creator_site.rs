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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET]);

    Router::new()
        .route("/", get(home))
        .route("/ai_songs/:id", get(show_ai_song))
        .route("/users/:username", get(show_ai_song_by_user))
        .route("/all_songs", get(show_ai_songs))
        .route("/all_users", get(show_users))
        .route("/charts", get(charts))
        .route("/music_videos", get(music_videos))
        .route("/current_song", get(show_current_song))
        .nest_service("/images", ServeDir::new("./tmp/music_videos"))
        .nest_service("/songs", ServeDir::new("./ai_songs"))
        .nest_service("/static", ServeDir::new("./static"))
        .with_state(state)
        .layer(cors)
}

type WebResult<T> = Result<T, (StatusCode, String)>;

async fn home(State(state): State<AppState>) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    let unplayed_songs = ai_playlist::get_unplayed_songs(&state.pool)
        .await
        .unwrap_or_default();
    let current_song = ai_playlist::get_current_song(&state.pool).await.ok();
    let current_song_info = ai_songs_vote::get_current_song_info(&state.pool)
        .await
        .unwrap_or(ai_songs_vote::CurrentSongInfo {
            current_song: None,
            votes_count: 0,
        });
    let previously_played_songs =
        ai_playlist::get_previously_played_songs(&state.pool)
            .await
            .unwrap_or_default();
    let (videos, image_scores) = ai_playlist::get_videos_and_image_scores(
        &state.pool,
        current_song_info.current_song.as_ref(),
    )
    .await;
    let users =
        ai_songs_vote::get_users_with_song_count_and_avg_score(&state.pool)
            .await
            .unwrap_or_default();
    let score = if let Some(song) = &current_song_info.current_song {
        ai_songs_vote::get_average_score(&state.pool, song.song_id)
            .await
            .map(|res| res.avg_score.to_string())
            .unwrap_or_else(|_| "No Votes".to_string())
    } else {
        "No Votes".to_string()
    };

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
        previously_played_songs,
    };

    render_template("home.html", context)
}

async fn show_users(State(state): State<AppState>) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    let songs = ai_playlist::all_songs(&state.pool)
        .await
        .map_err(internal_error)?;
    let users =
        ai_songs_vote::get_users_with_song_count_and_avg_score(&state.pool)
            .await
            .unwrap_or_default();

    let context = context! { songs, stats, users };

    render_template("users.html", context)
}

async fn charts(State(state): State<AppState>) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    let songs = ai_playlist::all_songs(&state.pool)
        .await
        .map_err(internal_error)?;
    let top_songs = ai_songs_vote::get_top_songs(&state.pool, 20)
        .await
        .unwrap_or(vec![]);

    let context = context! { songs, stats, top_songs };

    render_template("charts.html", context)
}

async fn music_videos(
    State(state): State<AppState>,
) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    let songs = ai_playlist::all_songs(&state.pool)
        .await
        .map_err(internal_error)?;

    let context = context! { songs, stats };

    render_template("music_videos.html", context)
}

async fn show_ai_songs(
    State(state): State<AppState>,
) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    //let songs = ai_playlist::all_songs(&state.pool)
    //    .await
    //    .map_err(internal_error)?;
    let songs = ai_playlist::all_songs_with_limit(&state.pool, 10)
        .await
        .map_err(internal_error)?;

    let context = context! { songs, stats };

    render_template("songs.html", context)
}

async fn show_ai_song_by_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> WebResult<Html<String>> {
    let songs = ai_playlist::models::get_songs_for_user(&state.pool, &username)
        .await
        .unwrap_or_default();
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;

    let context = context! { stats, songs };

    render_template("songs.html", context)
}

async fn show_current_song(
    State(state): State<AppState>,
) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;

    let current_song = ai_playlist::get_current_song(&state.pool).await.ok();
    let current_song_info = ai_songs_vote::get_current_song_info(&state.pool)
        .await
        .unwrap_or(ai_songs_vote::CurrentSongInfo {
            current_song: None,
            votes_count: 0,
        });

    let score = if let Some(song) = current_song_info.current_song {
        ai_songs_vote::get_average_score(&state.pool, song.song_id)
            .await
            .map(|res| res.avg_score.to_string())
            .unwrap_or_else(|_| "No Votes".to_string())
    } else {
        "No Votes".to_string()
    };

    let votes_count = current_song_info.votes_count;

    let context = context! {
        score,
        stats,
        current_song,
        votes_count,
    };

    render_template("current_song_banner.html", context)
}

async fn show_ai_song(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> WebResult<Html<String>> {
    let stats = ai_songs_vote::fetch_stats(&state.pool)
        .await
        .map_err(internal_error)?;
    let current_song =
        ai_playlist::find_song_by_id(&state.pool, &id.to_string())
            .await
            .map_err(not_found)?;
    let current_song_votes_count =
        ai_songs_vote::total_votes_by_id(&state.pool, current_song.song_id)
            .await
            .unwrap_or(0);
    let (videos, image_scores) = ai_playlist::get_videos_and_image_scores(
        &state.pool,
        Some(&current_song),
    )
    .await;

    let base_path = format!("/images/{}", current_song.song_id);

    let context = context! {
        stats,
        current_song,
        current_song_votes_count,
        videos,
        image_scores,
        base_path,
    };

    render_template("song.html", context)
}

fn render_template(
    template_name: &str,
    context: minijinja::value::Value,
) -> WebResult<Html<String>> {
    let tmpl = ENV.get_template(template_name).map_err(internal_error)?;
    let body = tmpl.render(context).map_err(internal_error)?;
    Ok(Html(body))
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn not_found<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, err.to_string())
}
