use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
//use sqlx::PgPool;
//use sqlx::PgPool;

use crate::{middleware::jwt::AuthenticatedUser, routes::AppState};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Songs {
    pub title: String,
    pub alternative_title: String,
    pub artist: String,
    pub artist_url: String,
    pub views: u64,
    pub image_src: String,
    //    pub image: Image,
    pub is_paused: bool,
    pub song_duration: u64,
    pub elapsed_seconds: u64,
    pub url: String,
    pub album: Option<String>,
    pub video_id: String,
    pub playlist_id: String,
    pub media_type: String,
    pub tags: Vec<String>,
}

pub async fn fetch_current_song(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Songs>, axum::http::StatusCode> {
    if user.uuid == state.config.devin_id || user.uuid == state.config.trin_id {
        let current_song_read = state.current_song.read().await;
        Ok(Json(current_song_read.clone()))
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

pub async fn post_current_song(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<Songs>,
) -> Result<String, axum::http::StatusCode> {
    // SIMPLE UPDATE/PUSH REQ IN SQL TO THE DATABASE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // would be made a req to when song skipped/next song
    if user.uuid == state.config.devin_id {
        let mut current_song_write = state.current_song.write().await;
        *current_song_write = req;
        drop(current_song_write);
        Ok("Updated Current Song".to_string())
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}
