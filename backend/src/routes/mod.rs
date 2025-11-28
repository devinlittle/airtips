use axum::{
    Router,
    //    routing::{delete, get, post},
    routing::{get, post},
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod songs;
use songs::Songs;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub current_song: Arc<RwLock<Songs>>,
}

pub fn create_routes(pool: PgPool) -> Router {
    let current_song_rw = Arc::new(RwLock::new(Songs {
        title: "App Started".to_string(),
        alternative_title: "Hi there".to_string(),
        artist: "Devin Little".to_string(),
        artist_url: "https://devinlittle.net".to_string(),
        views: 112918002,
        image_src: "https://devinlittle.net/proimgs/beatuifulpicture.png".to_string(),
        is_paused: false,
        song_duration: 181,
        elapsed_seconds: 160,
        url: "https://devinlittle.net".to_string(),
        album: None,
        video_id: "Devin Kirk".to_string(),
        playlist_id: "hi ther".to_string(),
        media_type: "CP".to_string(),
        tags: [
            "Devin Litte".to_string(),
            "デヴィン・リトル".to_string(),
            "Thank You".to_string(),
            "For looking at the source code".to_string(),
        ]
        .to_vec(),
    }));

    let app_state = AppState {
        pool,
        current_song: current_song_rw,
    };

    Router::new()
        .route("/fetch_song", get(songs::fetch_songs))
        .route("/post_song", post(songs::post_songs))
        .with_state(app_state)
}
