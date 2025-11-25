use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use worker::{console_error, Env, ObjectNamespace};

use crate::router::AppState;

pub fn get_relays_do(env: &Env) -> std::result::Result<ObjectNamespace, worker::Error> {
    env.durable_object("RELAYS")
}

#[derive(Debug, Clone)]
pub struct AppRelays(ObjectNamespace);

impl AppRelays {
    pub fn new(namespace: ObjectNamespace) -> Self {
        Self(namespace)
    }

    pub fn namespace(&self) -> &ObjectNamespace {
        &self.0
    }
}

impl FromRequestParts<AppState> for AppRelays {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> std::result::Result<Self, Self::Rejection> {
        let namespace = get_relays_do(&state.env()).map_err(|_| {
            console_error!("relays.do.init: failed to get relay durable object namespace");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(AppRelays::new(namespace))
    }
}
