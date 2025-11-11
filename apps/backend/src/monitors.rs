use axum::{
    routing::{get, post},
    Router,
};
use crate::router::AppState;

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(handlers::get_monitor_by_id))
        .route("/org/{org_id}", get(handlers::get_monitors_by_org_id))
        .route("/", post(handlers::create_monitor))
}
