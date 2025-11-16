use crate::router::AppState;
use axum::{Router, routing::get};

pub mod handlers;
pub mod service;
pub mod types;

// Most of these routes are nested under the /monitors route
pub fn router() -> Router<AppState> {
  Router::new()
    .route("/heartbeats", get(handlers::get_heartbeats_by_monitor_id_handler))
}