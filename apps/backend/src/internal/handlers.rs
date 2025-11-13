use crate::auth::current_user::CurrentUser;
use crate::cloudflare::ticker::ensure_all_tickers;
use crate::internal::types::{DispatchRequest, ReconcileResponse};
use crate::router::AppState;
use axum::{extract::State, http::HeaderMap, http::StatusCode, response::Result, Json};
use worker::console_error;

#[worker::send]
pub async fn reconcile_tickers_handler(
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

#[worker::send]
pub async fn dispatch_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DispatchRequest>,
) -> Result<StatusCode, StatusCode> {
    validate_dispatch_token(&state, &headers)?;
    handle_dispatch(state, payload).await?;

    Ok(StatusCode::ACCEPTED)
}

fn validate_dispatch_token(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let expected = state
        .env()
        .var("DISPATCH_TOKEN")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    match headers
        .get("x-dispatch-token")
        .and_then(|value| value.to_str().ok())
    {
        Some(actual) if actual == expected => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}