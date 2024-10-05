//use anyhow::Result;
//use axum::{
//    extract::{FromRef, Path, State},
//    http::{Method, StatusCode},
//    response::Html,
//    routing::get,
//    Router,
//};
//use std::fs;
//use std::sync::Arc;
//use subd_db::get_db_pool;
//
//use tower_http::{
//    cors::{Any, CorsLayer},
//    services::ServeDir,
//};
//use tracing_subscriber;
//
//#[derive(Clone, FromRef)]
//struct AppState {
//    pool: Arc<sqlx::PgPool>,
//}
//
//#[tokio::main]
//async fn main() {
//    tracing_subscriber::fmt::init();
//
//    let app = create_app().await;
//    let listener = tokio::net::TcpListener::bind("0.0.0.0:4001").await.unwrap();
//    axum::serve(listener, app).await.unwrap();
//}
//
//async fn create_app() -> Router {
//    let pool = get_db_pool().await;
//    let state = AppState {
//        pool: Arc::new(pool),
//    };
//
//    Router::new()
//        .route("/", get(root))
//        .route("/ai_songs/:id", get(show_ai_song))
//        .nest_service("/images", ServeDir::new("./tmp/music_videos"))
//        .with_state(state)
//        .layer(
//            CorsLayer::new()
//                .allow_origin(Any)
//                .allow_methods([Method::GET]),
//        )
//}
//
//async fn show_ai_song(
//    State(pool): State<Arc<sqlx::PgPool>>,
//    Path(id): Path<uuid::Uuid>,
//) -> Result<Html<String>, (StatusCode, String)> {
//    let mut html = String::new();
//    let ai_songs_count = ai_playlist::total_ai_songs(&pool).await.unwrap();
//    let ai_votes_count = ai_songs_vote::total_votes(&pool).await.unwrap();
//    let unplayed_songs_count =
//        ai_playlist::count_unplayed_songs(&pool).await.unwrap();
//    html.push_str(&header_html(
//        ai_songs_count,
//        ai_votes_count,
//        unplayed_songs_count,
//    ));
//    Ok(Html(format_song_html(html, id)))
//}
//
//fn format_song_html(mut html: String, id: uuid::Uuid) -> String {
//    html.push_str(&format!("SONG ID: {}", id));
//    html.push_str("<br />");
//    html.push_str(&"<a class=\"button\" href=\"javascript:window.history.back();\">Back</a>");
//    println!("In Show User!");
//    html
//}
//
//async fn root(
//    State(pool): State<Arc<sqlx::PgPool>>,
//) -> Result<Html<String>, (StatusCode, String)> {
//    let ai_songs_count = ai_playlist::total_ai_songs(&pool).await.unwrap();
//    let ai_votes_count = ai_songs_vote::total_votes(&pool).await.unwrap();
//    let unplayed_songs_count =
//        ai_playlist::count_unplayed_songs(&pool).await.unwrap();
//    let unplayed_songs = ai_playlist::get_unplayed_songs(&pool).await.unwrap();
//    let songs = ai_songs_vote::get_top_songs(&pool, 5).await.unwrap();
//
//    let current_song = ai_playlist::get_current_song(&pool).await;
//    let current_song_votes_count = match &current_song {
//        Ok(song) => ai_songs_vote::total_votes_by_id(&pool, song.song_id)
//            .await
//            .unwrap_or(0),
//        Err(_) => 0,
//    };
//
//    let html = generate_html(
//        pool,
//        ai_songs_count,
//        ai_votes_count,
//        current_song_votes_count,
//        unplayed_songs_count,
//        unplayed_songs,
//        songs,
//        current_song,
//    )
//    .await?;
//
//    Ok(Html(html))
//}
//
//async fn generate_html(
//    pool: Arc<sqlx::PgPool>,
//    ai_songs_count: i64,
//    ai_votes_count: i64,
//    current_songs_vote_count: i64,
//    unplayed_songs_count: i64,
//    unplayed_songs: Vec<ai_playlist::models::ai_songs::Model>,
//    songs: Vec<ai_songs_vote::AiSongRanking>,
//    current_song: Result<ai_playlist::models::ai_songs::Model, sqlx::Error>,
//) -> Result<String, (StatusCode, String)> {
//    let mut html = String::new();
//
//    for song in songs {
//        html.push_str(&format!(
//            "<a href=/ai_songs/{}>Song: {}</a>",
//            song.song_id, song.song_id
//        ));
//    }
//    html.push_str(&header_html(
//        ai_songs_count,
//        ai_votes_count,
//        unplayed_songs_count,
//    ));
//
//    html.push_str(&unplayed_songs_html(&unplayed_songs));
//    html.push_str(
//        &current_song_html(&pool, current_song, current_songs_vote_count)
//            .await?,
//    );
//    html.push_str("\n</body></html>");
//
//    Ok(html)
//}
//
//fn header_html(
//    ai_songs_count: i64,
//    ai_votes_count: i64,
//    unplayed_songs_count: i64,
//) -> String {
//    format!(
//        "<html><head><meta http-equiv=\"refresh\" content=\"5\" />
//        <style>
//            body {{
//                font-family: 'Comic Sans MS', cursive, sans-serif;
//            }}
//            .button {{
//                background-color: #04AA6D; /* Green */
//                border: none;
//                color: white;
//                padding: 15px 32px;
//                text-align: center;
//                text-decoration: none;
//                display: inline-block;
//                font-size: 16px;
//            }}
//            .sub-header{{
//                font-size: 200%;
//            }}
//            .current-song {{
//                padding: 10px;
//                border: 2px solid black;
//            }}
//            .header{{
//                font-size: 400%;
//            }}
//            .grid-container {{
//                display: grid;
//                grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
//                grid-gap: 10px;
//            }}
//            .grid-item {{
//                text-align: center;
//            }}
//            .full-width {{
//                width: 100%;
//                margin: 20px 0;
//                border: 0;
//                border-top: 1px solid #ccc;
//            }}
//        </style>
//        </head><body><h1 class=\"header grid-item\"> AI Top of the Pops</h1>
//        <h2 class=\"sub-header grid-item\"> Total AI Songs Created: {}</h2>
//        <h2 class=\"sub-header grid-item\"> Total AI Song Votes: {}</h2>
//        <h1 class=\"grid-item\">Songs in Playlist: {}</h1>
//        <h1 class=\"grid-item\"><code class=\"\">!vote 0.0 - 10.0</code></h1> ",
//        ai_songs_count, ai_votes_count, unplayed_songs_count
//    )
//}
////fn header_html(
////    ai_songs_count: i64,
////    ai_votes_count: i64,
////    unplayed_songs_count: i64,
////) -> String {
////    format!(
////        "<html><head><meta http-equiv=\"refresh\" content=\"5\" />
////        <style>
////            body {
////                font-family: 'Comic Sans MS', cursive, sans-serif;
////            }
////            .sub-header{
////                font-size: 200%;
////            }
////            .current-song {
////                padding: 10px;
////                border: 2px solid black;
////            }
////            .header{
////                font-size: 400%;
////            }
////            .grid-container {
////                display: grid;
////                grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
////                grid-gap: 10px;
////            }
////            .grid-item {
////                text-align: center;
////            }
////            .full-width {
////                width: 100%;
////                margin: 20px 0;
////                border: 0;
////                border-top: 1px solid #ccc;
////            }
////        </style>
////        </head><body><h1 class=\"header grid-item\"> AI Top of the Pops</h1>
////        <h2 class=\"sub-header grid-item\"> Total AI Songs Created: {}</h2>
////        <h2 class=\"sub-header grid-item\"> Total AI Song Votes: {}</h2>
////        <h1 class=\"grid-item\">Songs in Playlist: {}</h1>
////        <h1 class=\"grid-item\"><code class=\"\">!vote 0.0 - 10.0</code></h1> ",
////        ai_songs_count, ai_votes_count, unplayed_songs_count
////    )
////}
//
////fn header_html(
////    ai_songs_count: i64,
////    ai_votes_count: i64,
////    unplayed_songs_count: i64,
////) -> String {
////    format!(
////        "<html><head><meta http-equiv=\"refresh\" content=\"5\" />
////        <style>
////            body {
////                font-family: 'Comic Sans MS', cursive, sans-serif;
////            }
////            .sub-header{
////                font-size: 200%;
////            }
////            .current-song {
////                padding: 10px;
////                border: 2px solid black;
////            }
////            .header{
////                font-size: 400%;
////            }
////            .grid-container {
////                display: grid;
////                grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
////                grid-gap: 10px;
////            }
////            .grid-item {
////                text-align: center;
////            }
////            .full-width {
////                width: 100%;
////                margin: 20px 0;
////                border: 0;
////                border-top: 1px solid #ccc;
////            }
////        </style>
////        </head><body><h1 class=\"header grid-item\"> AI Top of the Pops</h1>
////        <h2 class=\"sub-header grid-item\"> Total AI Songs Created: {}</h2>
////        <h2 class=\"sub-header grid-item\"> Total AI Song Votes: {}</h2>
////        <h1 class=\"grid-item\">Songs in Playlist: {}</h1>
////        <h1 class=\"grid-item\"><code class=\"\">!vote 0.0 - 10.0</code></h1> ",
////        ai_songs_count, ai_votes_count, unplayed_songs_count
////    )
////}
//
//fn unplayed_songs_html(
//    unplayed_songs: &Vec<ai_playlist::models::ai_songs::Model>,
//) -> String {
//    let mut html = String::from("<h2>Songs in Playlist</h2>");
//    for song in unplayed_songs {
//        html.push_str(&format!(
//            "<div class=\"grid-item\">@{}'s {} - {}<div>",
//            song.username, song.title, song.song_id
//        ));
//    }
//    html.push_str("<hr />");
//    html
//}
//
//async fn current_song_html(
//    pool: &Arc<sqlx::PgPool>,
//    current_song: Result<ai_playlist::models::ai_songs::Model, sqlx::Error>,
//    current_songs_vote_count: i64,
//) -> Result<String, (StatusCode, String)> {
//    let mut html = String::new();
//
//    if let Ok(current_song) = current_song {
//        let score =
//            match ai_songs_vote::get_average_score(pool, current_song.song_id)
//                .await
//            {
//                Ok(score) => score.avg_score.to_string(),
//                Err(_) => "No Votes for Song".to_string(),
//            };
//        let music_directory =
//            format!("./tmp/music_videos/{}/", current_song.song_id);
//
//        if !fs::metadata(&music_directory).is_ok() {
//            fs::create_dir_all(&music_directory).map_err(|e| {
//                (
//                    StatusCode::INTERNAL_SERVER_ERROR,
//                    format!("Error creating directory: {}", e),
//                )
//            })?;
//        }
//
//        let images =
//            get_files_by_ext(&music_directory, &vec!["png", "jpg", "jpeg"]);
//        let videos = get_files_by_ext(&music_directory, &vec!["mp4"]);
//
//        let base_path = format!("/images/{}", current_song.song_id);
//        html.push_str(&format! (
//            "<h2 class=\"sub-header grid-item current-song\"> Current Song: {} | Tags: {} | Creator: @{} | {} | AVG Score: {} | Total Votes: {}</h2>",
//            current_song.title, current_song.tags, current_song.username, current_song.song_id, score, current_songs_vote_count
//        ));
//
//        html.push_str(&lyrics_html(&current_song.lyric));
//        html.push_str("<div class=\"grid-container\">");
//        html.push_str(&images_html(&base_path, &images));
//        html.push_str("<hr class=\"full-width\" />");
//        html.push_str(&videos_html(&base_path, &videos));
//    }
//
//    html.push_str("</div>");
//    Ok(html)
//}
//
//fn lyrics_html(lyrics: &Option<String>) -> String {
//    let lyrics = lyrics.clone().unwrap_or_default().replace("\n", "<br />");
//    format!("<div>{}</div>", lyrics)
//}
//
//fn images_html(base_path: &str, images: &Vec<String>) -> String {
//    images.iter().enumerate().map(|(index, image)| {
//        format! (
//            "<div class=\"grid-item\">
//                <img src=\"{}/{}\" alt=\"{}\" style=\"max-width:400px; max-height:400px;\" /><br/>
//                <h1><code>!like {} | !veto {}</code></h1>
//            </div>",
//            base_path, image, image, index, index
//        )
//    }).collect::<String>()
//}
//
//fn videos_html(base_path: &str, videos: &Vec<String>) -> String {
//    videos.iter().enumerate().map(|(index, video)| {
//        format! (
//            "<div class=\"grid-item\">
//                <video src=\"{}/{}\" alt=\"{}\" style=\"max-width:400px; max-height:400px;\" autoplay loop muted></video><br/>
//                <h1><code>!like {} | !veto {}</code></h1>
//            </div>",
//            base_path, video, video, index, index
//        )
//    }).collect::<String>()
//}
//
//fn get_files_by_ext(directory: &str, extensions: &[&str]) -> Vec<String> {
//    match fs::read_dir(directory) {
//        Ok(entries) => entries
//            .filter_map(|entry| {
//                let entry = entry.ok()?;
//                let path = entry.path();
//                if path.is_file() {
//                    if let Some(extension) = path.extension() {
//                        let ext = extension.to_string_lossy().to_lowercase();
//                        if extensions.contains(&ext.as_str()) {
//                            return path.file_name().map(|name| {
//                                name.to_string_lossy().into_owned()
//                            });
//                        }
//                    }
//                }
//                None
//            })
//            .collect(),
//        Err(_) => vec![],
//    }
//}

use anyhow::Result;
use axum::{
    extract::{FromRef, Path, State},
    http::{Method, StatusCode},
    response::Html,
    routing::get,
    Router,
};
use std::fs;
use std::sync::Arc;
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
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET]),
        )
}

async fn show_ai_song(
    State(pool): State<Arc<sqlx::PgPool>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Html<String>, (StatusCode, String)> {
    let stats = fetch_stats(&pool).await?;
    let mut html = header_html(&stats);
    html.push_str(&format!("SONG ID: {}<br />", id));
    html.push_str(&back_button_html());
    Ok(Html(html))
}

async fn root(
    State(pool): State<Arc<sqlx::PgPool>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let stats = fetch_stats(&pool).await?;
    let unplayed_songs = ai_playlist::get_unplayed_songs(&pool).await.unwrap();
    let top_songs = ai_songs_vote::get_top_songs(&pool, 5).await.unwrap();
    let current_song = ai_playlist::get_current_song(&pool).await;
    let current_song_votes_count = match &current_song {
        Ok(song) => ai_songs_vote::total_votes_by_id(&pool, song.song_id)
            .await
            .unwrap_or(0),
        Err(_) => 0,
    };

    let mut html = header_html(&stats);
    html.push_str(&unplayed_songs_html(&unplayed_songs));
    html.push_str(
        &current_song_html(&pool, current_song, current_song_votes_count)
            .await?,
    );
    html.push_str(&top_songs_html(&top_songs));
    Ok(Html(html))
}

async fn fetch_stats(
    pool: &Arc<sqlx::PgPool>,
) -> Result<Stats, (StatusCode, String)> {
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

struct Stats {
    ai_songs_count: i64,
    ai_votes_count: i64,
    unplayed_songs_count: i64,
}

fn header_html(stats: &Stats) -> String {
    format!(
        "<html><head><meta http-equiv=\"refresh\" content=\"5\" />
        <style>
            body {{
                font-family: 'Comic Sans MS', cursive, sans-serif;
                text-align: center;
            }}
            .button {{
                background-color: #04AA6D; /* Green */
                border: none;
                color: white;
                padding: 15px 32px;
                text-align: center;
                text-decoration: none;
                display: inline-block;
                font-size: 16px;
            }}
            .sub-header{{
                font-size: 200%;
            }}
            .current-song {{
                padding: 10px;
                border: 2px solid black;
            }}
            .header{{
                font-size: 400%;
            }}
            .grid-container {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
                grid-gap: 10px;
            }}
            .grid-item {{
                text-align: center;
            }}
            .full-width {{
                width: 100%;
                margin: 20px 0;
                border: 0;
                border-top: 1px solid #ccc;
            }}
        </style>
        </head><body><h1 class=\"header\">AI Top of the Pops</h1>
        <h2 class=\"sub-header\">Total AI Songs Created: {}</h2>
        <h2 class=\"sub-header\">Total AI Song Votes: {}</h2>
        <h1>Songs in Playlist: {}</h1>",
        stats.ai_songs_count, stats.ai_votes_count, stats.unplayed_songs_count
    )
}

fn back_button_html() -> String {
    "<a class=\"button\" href=\"javascript:window.history.back();\">Back</a>"
        .to_string()
}

fn unplayed_songs_html(
    unplayed_songs: &[ai_playlist::models::ai_songs::Model],
) -> String {
    let mut html = String::from("<h2>Songs in Playlist</h2>");
    for song in unplayed_songs {
        html.push_str(&format!(
            "<div class=\"grid-item\">@{}'s {} - {}<div>",
            song.username, song.title, song.song_id
        ));
    }
    html.push_str("<hr />");
    html
}

async fn current_song_html(
    pool: &Arc<sqlx::PgPool>,
    current_song: Result<ai_playlist::models::ai_songs::Model, sqlx::Error>,
    current_songs_vote_count: i64,
) -> Result<String, (StatusCode, String)> {
    if let Ok(current_song) = current_song {
        let score =
            ai_songs_vote::get_average_score(pool, current_song.song_id)
                .await
                .map(|s| s.avg_score.to_string())
                .unwrap_or_else(|_| "No Votes for Song".to_string());
        return Ok(format!(
            "<h2 class=\"sub-header current-song\">Current Song: {} | Tags: {} | Creator: @{} | {} | AVG Score: {} | Total Votes: {}</h2>",
            current_song.title, current_song.tags, current_song.username, current_song.song_id, score, current_songs_vote_count
        ));
    }
    Ok(String::new())
}

fn top_songs_html(songs: &[ai_songs_vote::AiSongRanking]) -> String {
    songs
        .iter()
        .map(|song| {
            format!(
                "<a href=/ai_songs/{}>Song: {}</a><br />",
                song.song_id, song.song_id
            )
        })
        .collect::<String>()
}
