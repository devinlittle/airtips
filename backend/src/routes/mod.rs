use axum::{
    Router,
    //    routing::{delete, get, post},
    routing::{get, post},
};
use sqlx::PgPool;

pub mod songs;

pub fn create_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/hello", get(songs::fetch_songs))
        .route("/post_song", post(songs::post_songs))
        .with_state(pool)
}
