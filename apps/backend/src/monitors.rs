use crate::{heartbeats, router::AppState};
use axum::{
    Router, routing::{delete, get, patch, post}
};

pub mod handlers;
pub mod service;
pub mod types;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(handlers::get_monitor_by_id_handler))
        .nest("/{id}", heartbeats::router())
        .route("/", get(handlers::get_monitors_handler))
        .route("/", post(handlers::create_monitor_handler))
        .route("/{id}", patch(handlers::update_monitor_handler))
        .route("/{id}", delete(handlers::delete_monitor_handler))
}
