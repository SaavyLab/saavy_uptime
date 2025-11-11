use crate::{monitors, organizations};
use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use worker::{send::SendWrapper, Env};

#[derive(Clone)]
pub struct AppState {
    env: SendWrapper<Env>,
}

impl AppState {
    pub fn new(env: Env) -> Self {
        Self {
            env: SendWrapper::new(env),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }
}

pub fn create_router(env: Env) -> Router {
    let app_state = AppState::new(env);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_router = Router::new()
        .nest("/monitors", monitors::router())
        .nest("/organizations", organizations::router())
        .layer(cors);

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api_router)
        .with_state(app_state)
}
