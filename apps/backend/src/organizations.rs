use crate::router::AppState;
use axum::{
    routing::{get, post},
    Router,
};

mod handlers;
mod types;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create_organization_handler))
        .route("/", get(handlers::get_organization_by_membership_handler))
        .route("/members", get(handlers::get_organization_members_handler))
}
