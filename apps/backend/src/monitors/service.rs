use axum::http::StatusCode;
use cuid2::create_id;
use worker::{console_error, wasm_bindgen::JsValue};

use crate::auth::membership::load_membership;
use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::cloudflare::d1::get_d1;
use crate::monitors::types::{CreateMonitor, Monitor};
use crate::router::AppState;
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

fn internal_error(context: &str, err: impl std::fmt::Debug) -> StatusCode {
    console_error!("{context}: {err:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn create_monitor(
    state: AppState,
    identity_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, StatusCode> {
    let membership = load_membership(&state, identity_id).await?;

    let d1 = get_d1(&state).map_err(|err| internal_error("create_monitor.d1", err))?;
    let statement = d1.prepare("INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)");
    let id = create_id().to_string();
    let now = now_ms();

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(&membership.organization_id),
        JsValue::from_str(&monitor.name),
        JsValue::from_str("http"),
        JsValue::from_str(&monitor.url),
        js_number(monitor.interval),
        js_number(monitor.timeout),
        js_number(monitor.follow_redirects as i64),
        js_number(monitor.verify_tls as i64),
        js_number(now),
        js_number(now),
    ];

    let query = statement
        .bind(&bind_values)
        .map_err(|err| internal_error("create_monitor.bind", err))?;

    query
        .run()
        .await
        .map_err(|err| internal_error("create_monitor.run", err))?;

    if let Err(err) = ensure_ticker_bootstrapped(&state.env(), &membership.organization_id).await {
        console_error!("create_monitor: ticker bootstrap failed: {err:?}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    get_monitor_by_id(state, id).await
}

pub async fn get_monitor_by_id(state: AppState, id: String) -> Result<Monitor, StatusCode> {
    let d1 = get_d1(&state).map_err(|err| internal_error("get_monitor_by_id.d1", err))?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE id = ?1");
    let query = statement
        .bind(&[id.into()])
        .map_err(|err| internal_error("get_monitor_by_id.bind", err))?;

    match query.first::<Monitor>(None).await {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => Err(internal_error("get_monitor_by_id.query", err)),
    }
}

pub async fn get_monitors(state: AppState, identity_id: &str) -> Result<Vec<Monitor>, StatusCode> {
    let membership = load_membership(&state, identity_id).await?;
    let d1 = get_d1(&state).map_err(|err| internal_error("get_monitors_by_org_id.d1", err))?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let query = statement
        .bind(&[membership.organization_id.into()])
        .map_err(|err| internal_error("get_monitors_by_org_id.bind", err))?;

    let all = query
        .all()
        .await
        .map_err(|err| internal_error("get_monitors_by_org_id.all", err))?;

    all.results::<Monitor>()
        .map_err(|err| internal_error("get_monitors_by_org_id.results", err))
}
