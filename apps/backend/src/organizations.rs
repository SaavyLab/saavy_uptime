use axum::{routing::{get, post}, Router};
use crate::router::AppState;

mod handlers;

pub use handlers::{create_organization, get_organization_by_id, CreateOrganization, Organization};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(handlers::get_organization_by_id))
        .route("/", post(handlers::create_organization))
}
