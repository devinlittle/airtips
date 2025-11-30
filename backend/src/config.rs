#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub devin_id: uuid::Uuid,
    pub trin_id: uuid::Uuid,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            jwt_secret: dotenvy::var("JWT_SECRET")?,
            devin_id: uuid::Uuid::parse_str(&dotenvy::var("devin_id")?)?,
            trin_id: uuid::Uuid::parse_str(&dotenvy::var("trin_id")?)?,
        })
    }
}
