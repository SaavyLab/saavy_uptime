use cuid2::create_id;
use std::result::Result;
use worker::wasm_bindgen::JsValue;

use crate::auth::membership::load_membership;
use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::cloudflare::d1::get_d1;
use crate::monitors::types::{CreateMonitor, Monitor, MonitorError};
use crate::router::AppState;
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

#[tracing::instrument(
    name = "monitors.create_for_member",
    skip(state, monitor),
    fields(identity_id = %identity_id)
)]
pub async fn create_monitor(
    state: &AppState,
    identity_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, MonitorError> {
    let membership = load_membership(&state, identity_id)
        .await
        .map_err(MonitorError::Membership)?;
    create_monitor_for_org(state, &membership.organization_id, monitor).await
}

#[tracing::instrument(
    name = "monitors.create_for_org",
    skip(state, monitor),
    fields(org_id = %org_id)
)]
pub async fn create_monitor_for_org(
    state: &AppState,
    org_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, MonitorError> {
    let d1 = get_d1(&state.env()).map_err(MonitorError::DbInit)?;
    let statement = d1.prepare("INSERT INTO monitors (id, org_id, name, kind, url, interval_s, timeout_ms, follow_redirects, verify_tls, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)");
    let id = create_id().to_string();
    let now = now_ms();

    let bind_values = vec![
        JsValue::from_str(&id),
        JsValue::from_str(org_id),
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

    let query = statement.bind(&bind_values).map_err(MonitorError::DbBind)?;

    query.run().await.map_err(MonitorError::DbRun)?;

    ensure_ticker_bootstrapped(&state.env(), org_id)
        .await
        .map_err(MonitorError::Bootstrap)?;

    get_monitor_by_id(&state, id).await
}

#[tracing::instrument(name = "monitors.get_by_id", skip(state), fields(monitor_id = %id))]
pub async fn get_monitor_by_id(state: &AppState, id: String) -> Result<Monitor, MonitorError> {
    let d1 = get_d1(&state.env()).map_err(|err| MonitorError::DbInit(err))?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE id = ?1");
    let query = statement
        .bind(&[id.into()])
        .map_err(|err| MonitorError::DbBind(err))?;

    match query.first::<Monitor>(None).await {
        Ok(Some(monitor)) => Ok(monitor),
        Ok(None) => Err(MonitorError::NotFound),
        Err(err) => Err(MonitorError::DbRun(err)),
    }
}

#[tracing::instrument(
    name = "monitors.list_for_member",
    skip(state),
    fields(identity_id = %identity_id)
)]
pub async fn get_monitors(
    state: &AppState,
    identity_id: &str,
) -> Result<Vec<Monitor>, MonitorError> {
    let membership = load_membership(&state, identity_id)
        .await
        .map_err(|err| MonitorError::Membership(err))?;
    let d1 = get_d1(&state.env()).map_err(|err| MonitorError::DbInit(err))?;
    let statement = d1.prepare("SELECT * FROM monitors WHERE org_id = ?1 ORDER BY created_at DESC");
    let query = statement
        .bind(&[membership.organization_id.into()])
        .map_err(|err| MonitorError::DbBind(err))?;

    let all = query.all().await.map_err(|err| MonitorError::DbRun(err))?;

    all.results::<Monitor>()
        .map_err(|err| MonitorError::DbRun(err))
}
