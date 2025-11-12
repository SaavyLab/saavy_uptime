use crate::{auth::jwt::AccessConfig, bootstrap, monitors, organizations};
use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use worker::{send::SendWrapper, Env};

#[derive(Clone)]
pub struct AppState {
    env: SendWrapper<Env>,
    access_config: AccessConfig,
}

impl AppState {
    pub fn new(env: Env, access_config: AccessConfig) -> Self {
        Self {
            env: SendWrapper::new(env),
            access_config,
        }
    }

    pub fn env(&self) -> Env {
        (*self.env).clone()
    }

    pub fn access_config(&self) -> &AccessConfig {
        &self.access_config
    }
}

pub fn create_router(env: Env) -> Router {
    let team_domain = env
        .var("ACCESS_TEAM_DOMAIN")
        .expect("ACCESS_TEAM_DOMAIN binding missing")
        .to_string();
    let audience = env
        .var("ACCESS_AUD")
        .expect("ACCESS_AUD binding missing")
        .to_string();

    let access_config = AccessConfig::new(team_domain, audience);
    let app_state = AppState::new(env, access_config);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_router = Router::new()
        .nest("/monitors", monitors::router())
        .nest("/organizations", organizations::router())
        .nest("/bootstrap", bootstrap::router())
        .layer(cors);

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api_router)
        .with_state(app_state)
}
