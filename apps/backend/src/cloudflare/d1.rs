use crate::router::AppState;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::result::Result;
use worker::{console_error, D1Database, Env};

pub fn get_d1(env: &Env) -> Result<D1Database, worker::Error> {
    env.d1("DB")
}

#[derive(Debug)]
pub struct AppDb(pub D1Database);

impl FromRequestParts<AppState> for AppDb {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let d1 = get_d1(&state.env()).map_err(|_| {
            console_error!("d1.init: failed to get d1 database");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(AppDb(d1))
    }
}
