use crate::router::AppState;
use axum::{
    routing::{get, post},
    Router,
};

mod handlers;
pub mod ticker_bootstrap;
pub mod types;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(handlers::status))
        .route("/initialize", post(handlers::initialize))
}
