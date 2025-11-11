use crate::{monitors, organizations};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use worker::Env;

#[derive(Clone)]
pub struct AppState {
    env: Env,
}

pub async fn create_router(env: Env) -> Router {
    let app_state = AppState { env };

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_router = Router::new()
        .nest("/monitors", monitors::router())
        .nest("/organizations", organizations::router())
        .layer(cors);

    Router::new()
        .get("/api/health", axum::routing::get(|| async { "ok" }))
        .nest("/api", api_router)
        .with_state(app_state)
}
