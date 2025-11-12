use crate::router::AppState;
use axum::{
    routing::{get, post},
    Router,
};

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(handlers::status))
        .route("/initialize", post(handlers::initialize))
}
