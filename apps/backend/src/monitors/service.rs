use cuid2::create_id;
use std::result::Result;
use worker::wasm_bindgen::JsValue;
use worker::{console_log, D1Database, ObjectNamespace};

use crate::bootstrap::ticker_bootstrap::ensure_ticker_bootstrapped;
use crate::d1c::queries::monitors::{create_monitor, get_monitor_by_id, update_monitor_status};
use crate::monitors::types::{
    CreateMonitor, HeartbeatResult, Monitor, MonitorError, MonitorStatusSnapshot, UpdateMonitor,
};
use crate::utils::date::now_ms;
use crate::utils::wasm_types::js_number;

#[tracing::instrument(
    name = "monitors.create_for_org",
    skip(d1, monitor),
    fields(org_id = %org_id)
)]
pub async fn create_monitor_for_org(
    ticker: &ObjectNamespace,
    d1: &D1Database,
    org_id: &str,
    monitor: CreateMonitor,
) -> Result<Monitor, MonitorError> {
    let id = create_id().to_string();
    create_monitor(
        &d1,
        &id,
        &org_id,
        &monitor.name.as_str(),
        "http",
        1,
        &monitor.config,
        "PENDING",
        now_ms(),
        now_ms(),
    )
    .await
    .map_err(MonitorError::DbRun)?;

    ensure_ticker_bootstrapped(ticker, org_id)
        .await
        .map_err(MonitorError::Bootstrap)?;

    match get_monitor_by_id(&d1, &id, &org_id).await {
        Ok(Some(monitor)) => Ok(monitor.into()),
        Ok(None) => Err(MonitorError::NotFound),
        Err(_) => Err(MonitorError::DbRun(worker::Error::RustError(
            "Failed to get monitor".to_string(),
        ))),
    }
}

#[tracing::instrument(
    name = "monitors.update_for_org",
    skip(d1),
    fields(org_id = %org_id, monitor_id = %monitor_id)
)]
pub async fn update_monitor_for_org(
    d1: &D1Database,
    org_id: &str,
    monitor_id: &str,
    monitor: UpdateMonitor,
) -> Result<(), MonitorError> {
    let mut fields = Vec::new();
    let mut values: Vec<JsValue> = Vec::new();

    console_log!("update monitor: {monitor:?}");

    if let Some(name) = monitor.name {
        fields.push("name = ?".to_string());
        values.push(JsValue::from_str(&name));
    }

    if let Some(kind) = monitor.kind {
        fields.push("kind = ?".to_string());
        values.push(JsValue::from_str(&kind.to_string()));
    }

    if let Some(config) = monitor.config {
        fields.push("url = ?".to_string());
        values.push(JsValue::from_str(
            &serde_json::to_string(&config).unwrap_or_default(),
        ));
    }

    if let Some(enabled) = monitor.enabled {
        fields.push("enabled = ?".to_string());
        values.push(js_number(enabled as i64));
    }

    if fields.is_empty() {
        return Err(MonitorError::NoFieldsToUpdate);
    }

    fields.push("updated_at = ?".to_string());
    values.push(js_number(now_ms()));

    values.push(JsValue::from_str(monitor_id));
    values.push(JsValue::from_str(org_id));

    let sql = format!(
        "UPDATE monitors SET {} WHERE id = ? AND org_id = ?",
        fields.join(", ")
    );

    let query = d1
        .prepare(&sql)
        .bind(&values)
        .map_err(MonitorError::DbBind)?;

    query.run().await.map_err(MonitorError::DbRun)?;

    console_log!("updated monitor: {monitor_id}");

    Ok(())
}

#[tracing::instrument(
    name = "monitors.update_status",
    skip(d1),
    fields(monitor_id = %heartbeat.monitor_id, org_id = %heartbeat.org_id, timestamp = %heartbeat.timestamp)
)]
pub async fn update_monitor_status_for_org(
    d1: &D1Database,
    heartbeat: &HeartbeatResult,
    snapshot: &MonitorStatusSnapshot,
) -> Result<(), MonitorError> {
    let now = now_ms();
    let first_checked_at = snapshot.first_checked_at.unwrap_or(now);
    let mut last_failed_at = snapshot.last_failed_at.unwrap_or_default();

    // We don't want to overwrite the last failed at if the monitor is already down.
    if heartbeat.status.is_down() && !snapshot.status.is_down() {
        last_failed_at = now;
    }

    let last_error = heartbeat.error.clone().unwrap_or_else(|| {
        if heartbeat.status.is_down() {
            "Health check failed".to_string()
        } else {
            String::new()
        }
    });

    update_monitor_status(
        &d1,
        heartbeat.status.to_string().as_str(),
        heartbeat.timestamp,
        last_failed_at,
        first_checked_at,
        heartbeat.latency_ms,
        last_error.as_str(),
        now,
        &heartbeat.monitor_id,
        &heartbeat.org_id,
    )
    .await
    .map_err(MonitorError::DbRun)?;

    Ok(())
}
