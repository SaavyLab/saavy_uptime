use crate::{bootstrap, heartbeats, internal, monitors, organizations};
use axum::{
    body::Body,
    http::Request,
    middleware::{from_fn, Next},
    routing::get,
    Router,
};
use hb_auth::{AuthConfig, HasAuthConfig};
use js_sys::Date;
use tower_http::cors::{Any, CorsLayer};
use tracing::field;
use worker::{send::SendWrapper, Env};

#[derive(Clone)]
pub struct AppState {
    env: SendWrapper<Env>,
    auth_config: AuthConfig,
}

impl AppState {
    pub fn new(env: &Env, auth_config: AuthConfig) -> Self {
        Self {
            env: SendWrapper::new(env.clone()),
            auth_config,
        }
    }

    pub fn env(&self) -> Env {
        (*self.env).clone()
    }

    pub fn access_config(&self) -> &AuthConfig {
        &self.auth_config
    }
}

impl HasAuthConfig for AppState {
    fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }
}

pub fn create_router(env: &Env) -> Router {
    let team_domain = env
        .var("ACCESS_TEAM_DOMAIN")
        .expect("ACCESS_TEAM_DOMAIN binding missing")
        .to_string();
    let audience = env
        .var("ACCESS_AUD")
        .expect("ACCESS_AUD binding missing")
        .to_string();

    let auth_config = AuthConfig::new(team_domain, audience);
    let app_state = AppState::new(&env, auth_config);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_router = Router::new()
        .nest("/heartbeats", heartbeats::router())
        .nest("/monitors", monitors::router())
        .nest("/organizations", organizations::router())
        .nest("/bootstrap", bootstrap::router())
        .nest("/internal", internal::router())
        .layer(cors)
        .layer(from_fn(trace_requests));

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api_router)
        .with_state(app_state)
}

async fn trace_requests(req: Request<Body>, next: Next) -> axum::response::Response {
    let request_id = cuid2::create_id();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let span = tracing::info_span!(
        "http.request",
        request_id = %request_id,
        method = %method,
        path = %path,
        status_code = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
        error = tracing::field::Empty
    );

    let _guard = span.enter();
    let start = Date::now();
    let response = next.run(req).await;
    let latency_ms = (Date::now() - start) as i64;
    let status_code = response.status().as_u16() as i64;

    span.record("latency_ms", &latency_ms);
    span.record("status_code", &status_code);

    if response.status().is_server_error() {
        let reason = response
            .status()
            .canonical_reason()
            .unwrap_or("server_error")
            .to_string();
        span.record("error", &field::display(&reason));
        tracing::error!(
            request_id = %request_id,
            %method,
            path = %path,
            status = status_code,
            latency_ms = latency_ms,
            reason = %reason,
            "response.error"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            %method,
            path = %path,
            status = status_code,
            latency_ms = latency_ms,
            "response.ready"
        );
    }

    response
}
