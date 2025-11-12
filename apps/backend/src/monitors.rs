use crate::router::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub mod handlers;
pub mod service;
pub mod types;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(handlers::get_monitor_by_id_handler))
        .route("/", get(handlers::get_monitors_handler))
        .route("/", post(handlers::create_monitor_handler))
}
