use axum::{extract::State, http::HeaderMap, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use worker::console_error;

use crate::{
    auth::current_user::CurrentUser,
    bootstrap::ticker_bootstrap::{ensure_all_tickers, TickerReconcileSummary},
    cloudflare::d1::get_d1,
    router::AppState,
    utils::{date::now_ms, wasm_types::js_number},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ticker/reconcile", post(reconcile_tickers))
        .route("/dispatch/run", post(run_dispatch))
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DispatchRequest {
    dispatch_id: String,
    monitor_id: String,
    scheduled_for_ts: i64,
}

#[worker::send]
async fn run_dispatch(
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

async fn handle_dispatch(state: AppState, payload: DispatchRequest) -> Result<(), StatusCode> {
    let d1 = get_d1(&state).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now = now_ms();

    let start_statement = d1.prepare(
        "UPDATE monitor_dispatches
         SET status = ?1,
             dispatched_at_ts = ?2
         WHERE id = ?3",
    );

    start_statement
        .bind(&[
            "running".into(),
            js_number(now),
            payload.dispatch_id.clone().into(),
        ])
        .map_err(|err| {
            console_error!("dispatch.bind.start failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .run()
        .await
        .map_err(|err| {
            console_error!("dispatch.update.start failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // TODO: Execute the actual monitor check. For now we mark as completed immediately.

    let complete_statement = d1.prepare(
        "UPDATE monitor_dispatches
         SET status = ?1,
             completed_at_ts = ?2
         WHERE id = ?3",
    );

    complete_statement
        .bind(&[
            "completed".into(),
            js_number(now),
            payload.dispatch_id.into(),
        ])
        .map_err(|err| {
            console_error!("dispatch.bind.complete failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .run()
        .await
        .map_err(|err| {
            console_error!("dispatch.update.complete failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}
