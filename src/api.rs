use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;

use crate::{payload::FraudRequest, state::AppState};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
        .with_state(state)
}

async fn ready() -> StatusCode {
    StatusCode::OK
}

#[derive(Serialize)]
struct FraudScoreResponse {
    approved: bool,
    fraud_score: f32,
}

async fn fraud_score(
    State(state): State<AppState>,
    Json(req): Json<FraudRequest>,
) -> Result<Json<FraudScoreResponse>, StatusCode> {
    let fraud_detector = &state.fraud_detector;

    let result = fraud_detector
        .analyze(&req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FraudScoreResponse {
        approved: result.approved,
        fraud_score: result.fraud_score,
    }))
}
