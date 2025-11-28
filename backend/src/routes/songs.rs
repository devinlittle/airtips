use axum::{Json, extract::State};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
//use serde_json::Value;
//use sqlx::PgPool;
use time::OffsetDateTime;

use crate::routes::AppState;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    username: String,
    #[serde(with = "jwt_numeric_date")]
    iat: OffsetDateTime,
    #[serde(with = "jwt_numeric_date")]
    exp: OffsetDateTime,
}

mod jwt_numeric_date {
    //! Custom serialization of OffsetDateTime to conform with the JWT spec (RFC 7519 section 2, "Numeric Date")
    use serde::{self, Deserialize, Deserializer, Serializer};
    use time::OffsetDateTime;

    /// Serializes an OffsetDateTime to a Unix timestamp (milliseconds since 1970/1/1T00:00:00T)
    pub fn serialize<S>(date: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.unix_timestamp();
        serializer.serialize_i64(timestamp)
    }

    /// Attempts to deserialize an i64 and use as a Unix timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        OffsetDateTime::from_unix_timestamp(i64::deserialize(deserializer)?)
            .map_err(|_| serde::de::Error::custom("invalid Unix timestamp value"))
    }
}

pub async fn fetch_songs(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
) -> Result<Json<Songs>, axum::http::StatusCode> {
    let jwt_secret = dotenvy::var("JWT_SECRET").unwrap();
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let uuid = match jsonwebtoken::decode::<Claims>(bearer.token(), &decoding_key, &validation)
        .map(|x| x.claims.sub)
    {
        Ok(uuid) => uuid,
        Err(err) => {
            let _ = match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    tracing::warn!("InvalidToken");
                    "Invalid Token"
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::warn!("InvalidSignature");
                    "Invalid Signature"
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::warn!("ExpiredSignature");
                    "Expiered Signature"
                }
                _ => {
                    tracing::warn!("Something really bad happened");
                    "Token Verifation fail"
                }
            };
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    let uuid =
        uuid::Uuid::parse_str(uuid.as_str()).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    if uuid.to_string() == dotenvy::var("trin_id").unwrap()
        || uuid.to_string() == dotenvy::var("devin_id").unwrap()
    {
        let current_song_read = state.current_song.read().await;
        Ok(Json(current_song_read.clone()))
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

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

pub async fn post_songs(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    State(state): State<AppState>,
    Json(req): Json<Songs>,
) -> Result<String, axum::http::StatusCode> {
    let jwt_secret = dotenvy::var("JWT_SECRET").unwrap();
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let uuid = match jsonwebtoken::decode::<Claims>(bearer.token(), &decoding_key, &validation)
        .map(|x| x.claims.sub)
    {
        Ok(uuid) => uuid,
        Err(err) => {
            let _ = match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    tracing::warn!("InvalidToken");
                    "Invalid Token"
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::warn!("InvalidSignature");
                    "Invalid Signature"
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::warn!("ExpiredSignature");
                    "Expiered Signature"
                }
                _ => {
                    tracing::warn!("Something really bad happened");
                    "Token Verifation fail"
                }
            };
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        }
    };

    let uuid =
        uuid::Uuid::parse_str(uuid.as_str()).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    if uuid.to_string() == dotenvy::var("devin_id").unwrap() {
        let mut current_song_write = state.current_song.write().await;
        *current_song_write = req;
        drop(current_song_write);
        Ok("Updated Current Song".to_string())
    } else {
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}
