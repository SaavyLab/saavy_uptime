use crate::{auth::jwt::AccessConfig, bootstrap, internal, monitors, organizations};
use axum::{routing::get, Router};
use http::{Request, Response};
use std::time::Duration;
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{field, Span};
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

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = cuid2::create_id();
            tracing::info_span!(
                "http.request",
                request_id = %request_id,
                method = %request.method(),
                path = %request.uri().path(),
                status_code = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
                error = tracing::field::Empty
            )
        })
        .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
            let status_code = response.status().as_u16() as i64;
            let latency_ms = latency.as_millis() as i64;
            span.record("status_code", &status_code);
            span.record("latency_ms", &latency_ms);
            tracing::info!(parent: span, "response.ready");
        })
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                let latency_ms = latency.as_millis() as i64;
                span.record("latency_ms", &latency_ms);
                let message = format!("{error:?}");
                let display = field::display(&message);
                span.record("error", &display);
                tracing::error!(parent: span, "response.error");
            },
        );

    let api_router = Router::new()
        .nest("/monitors", monitors::router())
        .nest("/organizations", organizations::router())
        .nest("/bootstrap", bootstrap::router())
        .nest("/internal", internal::router())
        .layer(cors)
        .layer(trace_layer);

    Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api_router)
        .with_state(app_state)
}
