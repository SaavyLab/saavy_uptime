use crate::router::AppState;
use axum::{
    routing::{get, post},
    Router,
};

mod handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}", get(handlers::get_organization_by_id_handler))
        .route("/", post(handlers::create_organization_handler))
}
