use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use worker::console_error;

use crate::{
    auth::current_user::CurrentUser,
    bootstrap::ticker_bootstrap::{ensure_all_tickers, TickerReconcileSummary},
    router::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/ticker/reconcile", post(reconcile_tickers))
}

#[derive(Serialize)]
struct ReconcileResponse {
    organizations: usize,
    bootstrapped: usize,
    failed: usize,
}

#[worker::send]
async fn reconcile_tickers(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> Result<Json<ReconcileResponse>, StatusCode> {
    match ensure_all_tickers(&state.env()).await {
        Ok(summary) => Ok(Json(ReconcileResponse {
            organizations: summary.organizations,
            bootstrapped: summary.bootstrapped,
            failed: summary.failed,
        })),
        Err(err) => {
            console_error!("ticker.reconcile: failed: {err:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
