use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use worker::AnalyticsEngineDataset;

use crate::router::AppState;

#[derive(Debug)]
pub struct AppAnalytics(pub AnalyticsEngineDataset);

impl FromRequestParts<AppState> for AppAnalytics {
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let dataset = state
            .env()
            .analytics_engine("AE_HEARTBEATS")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(AppAnalytics(dataset))
    }
}
