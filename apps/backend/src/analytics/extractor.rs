use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};

use crate::{analytics::client::AeQueryClient, router::AppState};

#[derive(Clone)]
pub struct AppAeClient(pub AeQueryClient);

impl FromRequestParts<AppState> for AppAeClient {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        AeQueryClient::from_env(&state.env())
            .await
            .map(AppAeClient)
            .map_err(|err| {
                worker::console_error!("analytics.client: {err:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}
