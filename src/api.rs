use axum::{Router, http::StatusCode, routing::get};

pub fn router() -> Router {
    Router::new().route("/ready", get(ready))
}

async fn ready() -> StatusCode {
    StatusCode::OK
}
