use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::routes::AppState;

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub uuid: uuid::Uuid,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    #[serde(with = "jwt_numeric_date")]
    pub iat: OffsetDateTime,
    #[serde(with = "jwt_numeric_date")]
    pub exp: OffsetDateTime,
}

pub async fn jwt_auth(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());

    let uuid_str = jsonwebtoken::decode::<Claims>(bearer.token(), &decoding_key, &validation)
        .map(|x| x.claims.sub)
        .map_err(|err| {
            match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    tracing::warn!("InvalidToken");
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::warn!("InvalidSignature");
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::warn!("ExpiredSignature");
                }
                _ => {
                    tracing::warn!("Token verification failed: {:?}", err);
                }
            }
            StatusCode::UNAUTHORIZED
        })?;

    let uuid = uuid::Uuid::parse_str(&uuid_str).map_err(|_| StatusCode::BAD_REQUEST)?;

    request.extensions_mut().insert(AuthenticatedUser { uuid });

    Ok(next.run(request).await)
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
