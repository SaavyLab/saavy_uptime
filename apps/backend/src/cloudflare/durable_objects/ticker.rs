use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use worker::{console_error, Env, ObjectNamespace};

use crate::router::AppState;

pub fn get_ticker_do(env: &Env) -> std::result::Result<ObjectNamespace, worker::Error> {
    env.durable_object("TICKER")
}

#[derive(Debug)]
pub struct AppTicker(pub ObjectNamespace);

impl FromRequestParts<AppState> for AppTicker {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let namespace = get_ticker_do(&state.env()).map_err(|_| {
            console_error!("ticker.do.init: failed to get ticker durable object");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(AppTicker(namespace))
    }
}
