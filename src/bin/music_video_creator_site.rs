use anyhow::Result;
use axum::{
    extract::FromRef, extract::State, http::StatusCode, routing::get, Router,
};
use axum::{http::Method, response::Html};
use chrono::prelude::*;
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

    let serve_dir = ServeDir::new("./tmp/music_videos");
    let pool = get_db_pool().await;
    let state = AppState {
        pool: Arc::new(pool),
    };

    let app = Router::new()
        .route("/", get(root))
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
    let ai_songs_count = ai_playlist::total_ai_songs(&pool).await.unwrap();
    let ai_votes_count = ai_songs_vote::total_votes(&pool).await.unwrap();
    let unplayed_songs_count =
        ai_playlist::count_unplayed_songs(&pool).await.unwrap();
    let unplayed_songs = ai_playlist::get_unplayed_songs(&pool).await.unwrap();

    let current_song = match ai_playlist::get_current_song(&pool).await {
        Ok(song) => song,
        Err(_) => {
            return Ok(Html(
                generate_html(
                    pool,
                    ai_songs_count,
                    ai_votes_count,
                    0,
                    unplayed_songs_count,
                    unplayed_songs,
                    Err("Error getting current song"),
                )
                .await?,
            ))
        }
    };

    let current_song_votes_count =
        match ai_songs_vote::total_votes_by_id(&pool, current_song.song_id)
            .await
        {
            Ok(count) => count,
            Err(_) => 0,
        };
    let html = generate_html(
        pool,
        ai_songs_count,
        ai_votes_count,
        current_song_votes_count,
        unplayed_songs_count,
        unplayed_songs,
        Ok(current_song),
    )
    .await?;

    Ok(Html(html))
}

async fn generate_html(
    pool: Arc<sqlx::PgPool>,
    ai_songs_count: i64,
    ai_votes_count: i64,
    current_songs_vote_count: i64,
    unplayed_songs_count: i64,
    unplayed_songs: Vec<ai_playlist::models::ai_songs::Model>,
    current_song: Result<ai_playlist::models::ai_songs::Model, &str>,
) -> Result<String, (StatusCode, String)> {
    let mut html = String::from(
        "<html>
            <head>
                <meta http-equiv=\"refresh\" content=\"5\" />
                <style>
                    body {
                        font-family: 'Comic Sans MS', cursive, sans-serif;
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
                    .full-width {
                        width: 100%;
                        margin: 20px 0;
                        border: 0;
                        border-top: 1px solid #ccc;
                    }
                </style>
            </head>
            <body>
        ",
    );

    html.push_str(&"<h1 class=\"header grid-item\"> AI Top of the Pops</h1>");
    html.push_str(&format!(
        "<h2 class=\"sub-header grid-item\"> Total AI Songs Created: {}</h2>",
        ai_songs_count
    ));
    html.push_str(&format!(
        "<h2 class=\"sub-header grid-item\"> Total AI Song Votes: {}</h2>",
        ai_votes_count,
    ));
    html.push_str(&format!(
        "<h1 class=\"grid-item\">Songs in Playlist: {}</h1>",
        unplayed_songs_count
    ));
    html.push_str(&"<h1 class=\"grid-item\"><code class=\"\">!vote 0.0 - 10.0</code></h1>");

    html.push_str(&"<h2>Songs in Playlist</h2>");
    for (_index, song) in unplayed_songs.into_iter().enumerate() {
        // TODO: Figure out why I'm too dumb to show a formatted string of a date
        // let pst = song.created_at.unwrap().hour();
        // This is wrong
        // let formatted =
        //     format!("{:?}", song.created_at.unwrap().format("%d/%m/%Y %H:%M"));
        //     format::strftime("%Y-%m-%d %H:%M:%S", &song.created_at.unwrap());
        // let pst_time = song.created_at.unwrap().format(&"%H:%M:%S").unwrap();
        html.push_str(&format!(
            "<div class=\"grid-item\">@{}'s {} - {}<div>",
            song.username, song.title, song.song_id
        ));
    }

    // We need iterate here
    html.push_str(&"<hr />");

    if let Ok(current_song) = current_song {
        let score =
            match ai_songs_vote::get_average_score(&pool, current_song.song_id)
                .await
            {
                Ok(score) => score.avg_score.to_string(),
                Err(_) => "No Votes for Song".to_string(),
            };
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

        let images =
            get_files_by_ext(&music_directory, &vec!["png", "jpg", "jpeg"]);
        let videos = get_files_by_ext(&music_directory, &vec!["mp4"]);

        let base_path = format!("/images/{}", current_song.song_id);
        html.push_str(&format!(
            "<h2 class=\"sub-header grid-item current-song\"> Current Song: {} | Tags: {} | Creator: @{} | {} | AVG Score: {} | Total Votes: {}</h2>",
            current_song.title, current_song.tags, current_song.username, current_song.song_id, score, current_songs_vote_count
        ));

        println!("--------------------");
        println!("Lyrics: {:?}", current_song.lyric);
        println!("--------------------");

        let lyrics = current_song
            .lyric
            .unwrap_or("".to_string())
            .replace("\n", "<br />");
        html.push_str(&format!("<div>{}</div>", lyrics));

        html.push_str("<div class=\"grid-container\">");

        for (index, image) in images.into_iter().enumerate() {
            html.push_str(&format!(
                "<div class=\"grid-item\">
                    <img src=\"{}/{}\" alt=\"{}\" style=\"max-width:400px; max-height:400px;\" /><br/>
                    <h1><code>!like {} | !veto {}</code></h1>
                </div>",
                base_path, image, image, index, index
            ));
        }

        html.push_str(&format!("<hr class=\"full-width\" />"));

        for (index, video) in videos.into_iter().enumerate() {
            html.push_str(&format!(
                "<div class=\"grid-item\">
                    <video src=\"{}/{}\" alt=\"{}\" style=\"max-width:400px; max-height:400px;\" autoplay loop muted></video><br/>
                    <h1><code>!like {} | !veto {}</code></h1>
                </div>",
                base_path, video, video, index, index
            ));
        }
    }

    html.push_str(
        "       </div>
            </body>
        </html>",
    );

    Ok(html)
}

fn get_files_by_ext(directory: &str, extensions: &[&str]) -> Vec<String> {
    let entries = match fs::read_dir(&directory) {
        Ok(d) => d,
        Err(_) => return vec![],
    };

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if extensions.contains(&ext.as_str()) {
                        return path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
}
