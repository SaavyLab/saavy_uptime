use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use worker::Cf;

use crate::router::AppState;

/// Extracts Cloudflare runtime metadata (colo, region, etc.) from the inbound request.
#[derive(Clone, Debug)]
pub struct RequestCf(pub Cf);

impl FromRequestParts<AppState> for RequestCf {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Cf>()
            .cloned()
            .map(RequestCf)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
