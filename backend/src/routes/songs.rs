use axum::{
    Extension, Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    middleware::jwt::AuthenticatedUser, middleware::jwt::jwt_numeric_date, routes::AppState,
};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongsFromClient {
    pub title: String,
    pub alternative_title: String,
    pub artist: String,
    pub artist_url: String,
    pub views: u64,
    pub image_src: String,
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

#[derive(Deserialize, Serialize, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SongHistory {
    pub title: String,
    pub alternative_title: String,
    pub artist: String,
    pub artist_url: String,
    pub image_src: String,
    pub song_duration: i64,
    pub url: String,
    pub album: Option<String>,
    pub video_id: String,
    pub playlist_id: String,
    pub media_type: String,
    pub tags: Vec<String>,
    #[serde(with = "jwt_numeric_date")]
    pub played_at: OffsetDateTime,
}

pub async fn fetch_current_song(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<SongsFromClient>, axum::http::StatusCode> {
    if user.uuid == state.config.devin_id || user.uuid == state.config.trin_id {
        let current_song_read = state.current_song.read().await;
        Ok(Json(current_song_read.clone()))
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

pub async fn post_song(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<SongsFromClient>,
) -> Result<String, axum::http::StatusCode> {
    if user.uuid == state.config.devin_id {
        let mut current_song_write = state.current_song.write().await;
        *current_song_write = req.clone();
        drop(current_song_write);

        sqlx::query!(
            r#"
            INSERT INTO song_history (user_id, title, alternative_title, artist, artist_url, image_src, song_duration, url, album, video_id, playlist_id, media_type, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
            user.uuid,
            req.title,
            req.alternative_title,
            req.artist,
            req.artist_url,
            req.image_src,
            req.song_duration as i64,
            req.url,
            req.album, // HACK: album will always return null because it is nullable...cant unwrap
                        // or expect need to fix.....LATER
            req.video_id,
            req.playlist_id,
            req.media_type,
            &req.tags,
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert song history: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok("Updated Current Song and Added To History".to_string())
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

#[derive(Serialize)]
pub struct PaginatedSongs {
    songs: Vec<SongHistory>,
    page: u32,
    total_pages: u32,
    has_more: bool,
}

async fn fetch_recent_song_history(
    pool: &PgPool,
    // user_uuid: &uuid::Uuid,
    limit: i64,
) -> Result<Vec<SongHistory>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            title,
            alternative_title,
            artist,
            artist_url,
            image_src,
            song_duration,
            url,
            album as "album?",
            video_id,
            playlist_id,
            media_type,
            tags,
            played_at
        FROM song_history
        ORDER BY played_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    // Manual mapping with conversion
    let songs = rows
        .into_iter()
        .map(|row| SongHistory {
            title: row.title,
            alternative_title: row.alternative_title,
            artist: row.artist,
            artist_url: row.artist_url,
            image_src: row.image_src,
            song_duration: row.song_duration,
            url: row.url,
            album: row.album,
            video_id: row.video_id,
            playlist_id: row.playlist_id,
            media_type: row.media_type,
            tags: row.tags,
            played_at: row.played_at.assume_utc(),
        })
        .collect();

    Ok(songs)
}

pub async fn recently_played(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(page): Path<u32>,
) -> Result<Json<PaginatedSongs>, axum::http::StatusCode> {
    if user.uuid == state.config.devin_id || user.uuid == state.config.trin_id {
        const PAGE_SIZE: usize = 50;

        let all_songs = fetch_recent_song_history(&state.pool, 150)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        let total_songs = all_songs.len();
        let total_pages = (total_songs + PAGE_SIZE - 1) / PAGE_SIZE;
        let start = (page.saturating_sub(1) as usize) * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(total_songs);

        if start >= total_songs {
            return Err(axum::http::StatusCode::NOT_FOUND);
        }

        let songs = all_songs[start..end].to_vec();

        Ok(Json(PaginatedSongs {
            songs,
            page,
            total_pages: total_pages as u32,
            has_more: page < total_pages as u32,
        }))
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}
