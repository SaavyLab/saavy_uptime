use std::ops::Deref;

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use worker::Cf;

/// Extracts Cloudflare runtime metadata (colo, region, etc.) from the inbound request.
#[derive(Clone, Debug)]
pub struct RequestCf(pub Cf);

impl RequestCf {
    pub fn into_inner(self) -> Cf {
        self.0
    }
}

impl Deref for RequestCf {
    type Target = Cf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequestCf
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Cf>()
            .cloned()
            .map(RequestCf)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
