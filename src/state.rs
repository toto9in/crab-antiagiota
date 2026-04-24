use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::fraud_detector::FraudDetector;

#[derive(Clone)]
pub struct AppState {
    pub fraud_detector: FraudDetector,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            fraud_detector: FraudDetector::new(pool),
        }
    }

    pub async fn from_env() -> Result<Self, sqlx::Error> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://rinha2026:rinha2026@localhost:55432/rinha2026".into());

        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await?;

        Ok(Self::new(pool))
    }
}
